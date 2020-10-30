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
    id: WidgetId,
}

impl TabController {
    pub fn new(id: WidgetId) -> TabController {
        TabController {
            grpc_client: None,
            id,
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
        if old_data.address.get() != data.address.get() {
            if let Some(uri) = data.address.get() {
                self.grpc_client = grpc::Client::new(uri.clone()).ok();
            } else {
                self.grpc_client = None;
            }
        }

        child.update(ctx, old_data, data, env)
    }
}

impl TabController {
    fn command(&mut self, ctx: &mut EventCtx, command: &Command, data: &mut TabState) -> Handled {
        if command.is(command::START_SEND) {
            if let (Some(grpc_client), Some(request)) = (&self.grpc_client, data.request.get()) {
                let event_sink = ctx.get_external_handle();
                let target = Target::Widget(self.id);
                grpc_client.send(request.clone(), move |response| {
                    event_sink
                        .submit_command(command::FINISH_SEND, SingleUse::new(response), target)
                        .ok();
                });
                data.request_state = RequestState::Active;
            }
            Handled::Yes
        } else if let Some(response) = command.get(command::FINISH_SEND) {
            let result = response.take().expect("response already handled");
            data.response.update(result);
            data.request_state = RequestState::NotStarted;
            Handled::Yes
        } else {
            Handled::No
        }
    }
}
