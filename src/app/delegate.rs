use druid::{AppDelegate, ExtEventSink, SingleUse, Target};

use crate::app::{self, command};
use crate::grpc;

pub(in crate::app) fn build(event_sink: ExtEventSink) -> impl AppDelegate<app::State> {
    Delegate {
        event_sink,
        grpc_client: grpc::Client::new(),
    }
}

struct Delegate {
    event_sink: ExtEventSink,
    grpc_client: grpc::Client,
}

impl AppDelegate<app::State> for Delegate {
    fn command(
        &mut self,
        _ctx: &mut druid::DelegateCtx,
        _target: druid::Target,
        cmd: &druid::Command,
        data: &mut app::State,
        _env: &druid::Env,
    ) -> bool {
        log::info!("Received command: {:?}", cmd);
        if let Some(file) = cmd.get(druid::commands::OPEN_FILE) {
            if let Err(err) = data.sidebar.add_from_path(file.path()) {
                log::error!("Error loading file: {:?}", err);
            }
            false
        } else if cmd.is(command::START_SEND) {
            let event_sink = self.event_sink.clone();
            self.grpc_client
                .send(data.request.request(), move |response| {
                    event_sink
                        .submit_command(
                            command::FINISH_SEND,
                            SingleUse::new(response),
                            Target::Global,
                        )
                        .ok();
                });
            false
        } else if let Some(response) = cmd.get(command::FINISH_SEND) {
            let result = response.take().expect("response already handled");
            data.response.update(result);
            false
        } else {
            true
        }
    }
}
