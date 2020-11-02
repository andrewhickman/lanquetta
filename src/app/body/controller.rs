use druid::{
    widget::{prelude::*, Controller},
    Command, Handled, SingleUse, Target,
};

use crate::{
    app::{
        body::{RequestState, TabState},
        command,
    },
    grpc,
};

pub struct TabController {
    grpc_client: Option<grpc::Client>,
}

impl TabController {
    pub fn new() -> TabController {
        TabController { grpc_client: None }
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
}

impl TabController {
    fn command(&mut self, ctx: &mut EventCtx, command: &Command, data: &mut TabState) -> Handled {
        log::info!("Body received command: {:?}", command);

        if command.is(command::START_CONNECT) {
            if let Some(uri) = data.address.uri() {
                self.grpc_client = None;
                let event_sink = ctx.get_external_handle();
                let target = Target::Widget(ctx.widget_id());
                grpc::Client::new(uri.clone(), move |result| {
                    event_sink
                        .submit_command(command::FINISH_CONNECT, SingleUse::new(result), target)
                        .ok();
                });
                data.address
                    .set_request_state(RequestState::ConnectInProgress);
            }
            Handled::Yes
        } else if let Some(result) = command.get(command::FINISH_CONNECT) {
            let (uri, result) = result.take().unwrap();
            if data.address.uri() != Some(&uri) {
                return Handled::Yes;
            }

            self.grpc_client = match result {
                Ok(client) => {
                    data.address.set_request_state(RequestState::Connected);
                    Some(client)
                }
                Err(err) => {
                    log::error!("Connect failed {:?}", err);
                    data.address.set_request_state(RequestState::ConnectFailed);
                    // TODO
                    None
                }
            };
            Handled::Yes
        } else if command.is(command::START_SEND) {
            if self.grpc_client.is_none() {
                if let Some(uri) = data.address.uri() {
                    self.grpc_client = Some(grpc::Client::new_lazy(uri.clone()));
                    ctx.submit_command(command::START_CONNECT.with(uri.clone()).to(ctx.widget_id()));
                }
            }

            if let (Some(grpc_client), Some(request)) = (&self.grpc_client, data.request.get()) {
                let event_sink = ctx.get_external_handle();
                let target = Target::Widget(ctx.widget_id());
                grpc_client.send(request.clone(), move |response| {
                    event_sink
                        .submit_command(command::FINISH_SEND, SingleUse::new(response), target)
                        .ok();
                });
                data.address.set_request_state(RequestState::Active);
            }
            Handled::Yes
        } else if let Some(response) = command.get(command::FINISH_SEND) {
            let result = response.take().expect("response already handled");
            data.response.update(result);
            Handled::Yes
        } else {
            Handled::No
        }
    }
}
