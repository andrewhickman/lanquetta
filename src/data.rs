use druid::{Data, Lens};

use crate::channel::ChannelState;

/// The top-level application state.
#[derive(Clone, Data, Lens)]
pub(crate) struct AppState {
    pub name: String,
    pub channel: ChannelState,
}

impl AppState {
    pub fn new() -> Self {
        AppState {
            name: "hello".to_owned(),
            channel: ChannelState::new(),
        }
    }
}
