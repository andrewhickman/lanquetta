use druid::{AppDelegate, Command, DelegateCtx, Env, ExtEventSink, Handled, Target};

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
        _ctx: &mut DelegateCtx,
        _target: Target,
        cmd: &Command,
        data: &mut app::State,
        _env: &Env,
    ) -> Handled {
        log::info!("Received command: {:?}", cmd);
        if let Some(file) = cmd.get(druid::commands::OPEN_FILE) {
            if let Err(err) = data.sidebar.add_from_path(file.path()) {
                log::error!("Error loading file: {:?}", err);
            }
            Handled::Yes
        } else if let Some(method) = cmd.get(command::SELECT_OR_CREATE_TAB) {
            data.body.select_or_create_tab(method.clone());
            Handled::Yes
        } else if let Some(method) = cmd.get(command::CREATE_TAB) {
            data.body.create_tab(method.clone());
            Handled::Yes
        } else {
            Handled::No
        }
    }
}
