use druid::{AppDelegate, Command, DelegateCtx, Env, Handled, Target};

use crate::app::{self, command};

pub(in crate::app) fn build() -> impl AppDelegate<app::State> {
    Delegate
}

struct Delegate;

impl AppDelegate<app::State> for Delegate {
    fn command(
        &mut self,
        _ctx: &mut DelegateCtx,
        _target: Target,
        cmd: &Command,
        data: &mut app::State,
        _env: &Env,
    ) -> Handled {
        tracing::debug!("Received command: {:?}", cmd);
        if let Some(file) = cmd.get(druid::commands::OPEN_FILE) {
            if let Err(err) = data.sidebar.add_from_path(file.path()) {
                data.error = Some(format!("Error loading file: {:?}", err));
            } else {
                data.error = None;
            }
            Handled::Yes
        } else if cmd.is(command::OPEN_GITHUB) {
            let _ = open::that_in_background(concat!(
                "https://github.com/andrewhickman/grpc-client/tree/",
                env!("VERGEN_GIT_SHA")
            ));
            Handled::Yes
        } else if cmd.is(command::CLOSE_SELECTED_TAB) {
            data.body.close_selected_tab();
            Handled::Yes
        } else if cmd.is(command::SELECT_NEXT_TAB) {
            data.body.select_next_tab();
            Handled::Yes
        } else if cmd.is(command::SELECT_PREV_TAB) {
            data.body.select_prev_tab();
            Handled::Yes
        } else if cmd.is(command::CLEAR) {
            data.body.clear_request_history();
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
