use druid::{
    widget::{prelude::*, Controller},
    Command, Handled, Selector, SingleUse,
};

use crate::{
    app::{
        body::{RequestState, TabState},
        command,
    },
    grpc,
};

pub struct TabController {
    client: Option<grpc::Client>,
    call: Option<grpc::Call>,
}

impl TabController {
    pub fn new() -> TabController {
        TabController {
            client: None,
            call: None,
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

const FINISH_CONNECT: Selector<grpc::ConnectResult> = Selector::new("app.body.finish-connect");
const DISCONNECT: Selector = Selector::new("app.body.disconnect");
const FINISH_SEND: Selector<SingleUse<Option<grpc::ResponseResult>>> =
    Selector::new("app.body.finish-send");

impl TabController {
    fn command(&mut self, ctx: &mut EventCtx, command: &Command, data: &mut TabState) -> Handled {
        log::debug!("Body received command: {:?}", command);

        if command.is(command::CONNECT) {
            self.start_connect(ctx, data);
            Handled::Yes
        } else if let Some(result) = command.get(FINISH_CONNECT) {
            self.finish_connect(ctx, data, result.clone());
            Handled::Yes
        } else if command.is(command::SEND) {
            self.start_send(ctx, data);
            Handled::Yes
        } else if let Some(response) = command.get(FINISH_SEND) {
            self.finish_send(ctx, data, response.take().unwrap());
            Handled::Yes
        } else if command.is(DISCONNECT) {
            self.disconnect(ctx, data);
            Handled::Yes
        } else {
            Handled::No
        }
    }
}

impl TabController {
    fn start_connect(&mut self, ctx: &mut EventCtx, data: &mut TabState) {
        let uri = match data.address.uri() {
            Some(uri) => uri.clone(),
            None => {
                log::error!("Connect called with no address");
                return;
            }
        };

        if self.is_connected(data) {
            return;
        }

        let event_sink = ctx.get_external_handle();
        let target = ctx.widget_id();

        tokio::spawn(async move {
            let client_result = grpc::Client::new(uri).await;
            let _ = event_sink.submit_command(FINISH_CONNECT, client_result, target);
        });

        if !self.is_active() {
            data.address
                .set_request_state(RequestState::ConnectInProgress);
        }
    }

    fn finish_connect(
        &mut self,
        _: &mut EventCtx,
        data: &mut TabState,
        result: grpc::ConnectResult,
    ) {
        match result {
            Ok(client) if Some(client.uri()) == data.address.uri() => {
                self.client = Some(client);

                self.set_request_state(data);
            }
            Err((uri, _)) if Some(&uri) == data.address.uri() => {
                data.address.set_request_state(RequestState::ConnectFailed);
            }
            _ => (),
        }
    }

    fn start_send(&mut self, ctx: &mut EventCtx, data: &mut TabState) {
        let (uri, request) =
            if let (Some(uri), Some(request)) = (data.address.uri(), data.request().get()) {
                (uri.clone(), request.clone())
            } else {
                log::error!("Send called with no address/request");
                return;
            };

        data.stream.add_request(&request);
        
        if let Some(call) = &self.call {
            call.send(request);
        } else {
            let client = match &self.client {
                Some(client) if client.uri() == &uri => client.clone(),
                _ => {
                    log::error!("Send called with invalid client");
                    return;
                }
            };

            let event_sink = ctx.get_external_handle();
            let target = ctx.widget_id();

            self.call = Some(client.call(&data.method, request, move |response| {
                let _ = event_sink.submit_command(FINISH_SEND, SingleUse::new(response), target);
            }));

            data.address.set_request_state(RequestState::Active);
        }
    }

    fn finish_send(
        &mut self,
        _: &mut EventCtx,
        data: &mut TabState,
        response: Option<grpc::ResponseResult>,
    ) {
        match response {
            Some(result) => {
                data.stream.add_response(&result);
            }
            None => {
                self.call = None;
            }
        }

        self.set_request_state(data);
    }

    fn disconnect(&mut self, _: &mut EventCtx, data: &mut TabState) {
        self.client = None;

        self.set_request_state(data);
    }

    fn is_connected(&self, data: &mut TabState) -> bool {
        match &self.client {
            Some(client) if data.address.uri() == Some(client.uri()) => true,
            _ => false,
        }
    }

    fn is_active(&self) -> bool {
        self.call.is_some()
    }

    fn set_request_state(&self, data: &mut TabState) {
        let request_state = match (self.is_active(), self.is_connected(data)) {
            (false, false) => RequestState::NotStarted,
            (false, true) => RequestState::Connected,
            (true, _) => RequestState::Active,
        };
        data.address.set_request_state(request_state);
    }
}
