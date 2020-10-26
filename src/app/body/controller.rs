use druid::{Command, Handled, SingleUse, Target, widget::{Controller, prelude::*}};

use crate::{app::{body::TabState, command}, grpc};

pub struct TabController {
    grpc_client: grpc::Client,
    id: WidgetId,
}

impl TabController {
    pub fn new(id: WidgetId) -> TabController {
        TabController {
            grpc_client: grpc::Client::new(),
            id,
        }
    }
}

impl<W> Controller<TabState, W> for TabController where W: Widget<TabState> {
    fn event(&mut self, child: &mut W, ctx: &mut EventCtx, event: &Event, data: &mut TabState, env: &Env) {
        match event {
            Event::Command(command) if self.command(ctx, command, data) == Handled::Yes => (),
            _ => child.event(ctx, event, data, env)
        }
    }
}

impl TabController {
    fn command(&mut self, ctx: &mut EventCtx, command: &Command, data: &mut TabState) -> Handled {
        if command.is(command::START_SEND) {
            if let Some(address) = data.address.get() {
                let event_sink = ctx.get_external_handle();
                let target = Target::Widget(self.id);
                self.grpc_client
                    .send(&address, data.request.get(), move |response| {
                        event_sink
                            .submit_command(
                                command::FINISH_SEND,
                                SingleUse::new(response),
                                target,
                            )
                            .ok();
                    });
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
