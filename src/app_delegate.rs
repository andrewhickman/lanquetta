use druid::{AppDelegate, Command, DelegateCtx, Env, Target};

use crate::data::AppState;

#[derive(Debug, Default)]
pub struct Delegate;

impl AppDelegate<AppState> for Delegate {
    fn command(
        &mut self,
        _ctx: &mut DelegateCtx,
        _target: Target,
        _cmd: &Command,
        _data: &mut AppState,
        _env: &Env,
    ) -> bool {
        true
    }
}
