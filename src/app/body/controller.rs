mod client;

use druid::{
    widget::{prelude::*, Controller},
    Command, Handled, Selector, SingleUse,
};
use http::Uri;

use self::client::ClientState;
use crate::{
    app::{
        body::{RequestState, TabState},
        command,
    },
    grpc,
};

pub struct TabController {
    client: ClientState,
    active: bool,
}

impl TabController {
    pub fn new() -> TabController {
        TabController {
            client: ClientState::new(),
            active: false,
        }
    }
}

impl<W> Controller<TabState, W> for TabController
where
    W: Widget<TabState>,
{
    fn event(
        &mut self,
        child: &mut W,
        ctx: &mut EventCtx,
        event: &Event,
        data: &mut TabState,
        env: &Env,
    ) {
        match event {
            Event::Command(command) if self.command(ctx, command, data) == Handled::Yes => (),
            _ => child.event(ctx, event, data, env),
        }
    }

    fn update(
        &mut self,
        child: &mut W,
        ctx: &mut UpdateCtx,
        old_data: &TabState,
        data: &TabState,
        env: &Env,
    ) {
        if old_data.address.uri() != data.address.uri() {
            ctx.submit_command(DISCONNECT.to(ctx.widget_id()));
        }

        child.update(ctx, old_data, data, env)
    }
}

const FINISH_CONNECT: Selector<(Uri, grpc::ConnectResult)> =
    Selector::new("app.body.finish-connect");
const DISCONNECT: Selector = Selector::new("app.body.disconnect");
const FINISH_SEND: Selector<SingleUse<grpc::ResponseResult>> =
    Selector::new("app.body.finish-send");

impl TabController {
    fn command(&mut self, ctx: &mut EventCtx, command: &Command, data: &mut TabState) -> Handled {
        log::debug!("Body received command: {:?}", command);

        if command.is(command::CONNECT) {
            if let Some(uri) = data.address.uri() {
                self.start_connect(ctx, uri.clone());
                self.update_request_state(data);
            } else {
                log::error!("Connect called with no address");
            }
            Handled::Yes
        } else if let Some((uri, result)) = command.get(FINISH_CONNECT) {
            self.finish_connect(uri, result.clone());
            self.update_request_state(data);
            Handled::Yes
        } else if command.is(command::SEND) {
            if let (Some(uri), Some(request)) = (data.address.uri(), data.request.get()) {
                self.start_send(ctx, uri.clone(), request.clone());
                self.update_request_state(data);
            } else {
                log::error!("Send called with no address/request");
            }
            Handled::Yes
        } else if let Some(response) = command.get(FINISH_SEND) {
            self.finish_send();
            data.response.update(response.take().unwrap());
            self.update_request_state(data);
            Handled::Yes
        } else if command.is(DISCONNECT) {
            self.client.reset();
            self.update_request_state(data);
            Handled::Yes
        } else {
            Handled::No
        }
    }
}

impl TabController {
    fn start_connect(&mut self, ctx: &mut EventCtx, uri: Uri) {
        let receiver = self.client.get(&uri);

        let event_sink = ctx.get_external_handle();
        let target = ctx.widget_id();
        tokio::spawn(async move {
            let result = receiver.borrow().await.expect("connect did not complete");
            let _ = event_sink.submit_command(FINISH_CONNECT, (uri, result.clone()), target);
        });
    }

    fn finish_connect(&mut self, uri: &Uri, result: grpc::ConnectResult) {
        self.client.set(uri, result);
    }

    fn start_send(&mut self, ctx: &mut EventCtx, uri: Uri, request: grpc::Request) {
        if self.active {
            log::error!("Send started while active");
            return;
        }
        self.active = true;

        let receiver = self.client.get(&uri);

        let event_sink = ctx.get_external_handle();
        let target = ctx.widget_id();

        tokio::spawn(async move {
            let client_result = receiver.borrow().await.unwrap().clone();

            let _ = event_sink.submit_command(FINISH_CONNECT, (uri, client_result.clone()), target);

            let send_result = match client_result {
                Ok(client) => client.send(request).await,
                Err(err) => Err(err),
            };

            let _ = event_sink.submit_command(FINISH_SEND, SingleUse::new(send_result), target);
        });
    }

    fn finish_send(&mut self) {
        if !self.active {
            log::error!("Send finished while not active");
            return;
        }
        self.active = false;
    }

    fn update_request_state(&mut self, data: &mut TabState) {
        let request_state = match (self.active, &self.client) {
            (false, ClientState::NotConnected { .. }) => RequestState::NotStarted,
            (false, ClientState::ConnectInProgress { .. }) => RequestState::ConnectInProgress,
            (false, ClientState::Connected { .. }) => RequestState::Connected,
            (false, ClientState::ConnectFailed { .. }) => RequestState::ConnectFailed,
            (true, _) => RequestState::Active,
        };

        data.address.set_request_state(request_state);
    }
}
