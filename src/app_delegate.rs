use druid::{AppDelegate, Command, DelegateCtx, Env, Target};

use crate::command;
use crate::data::AppState;

#[derive(Debug, Default)]
pub struct Delegate;

impl AppDelegate<AppState> for Delegate {
    fn command(
        &mut self,
        _ctx: &mut DelegateCtx,
        _target: Target,
        cmd: &Command,
        data: &mut AppState,
        _env: &Env,
    ) -> bool {
        println!("{:?}", cmd);

        if let Some((address, tls)) = cmd.get(command::CONNECT) {
            if let Err(err) = data.connection.connect(&address, *tls) {
                log::error!("error: {}", err);
            }
        }

        true
    }
}
