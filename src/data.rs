use druid::{Data, Lens};

/// The top-level application state.
#[derive(Clone, Data, Lens)]
pub(crate) struct AppState {
    pub name: String,
}

impl AppState {
    pub fn new() -> Self {
        AppState {
            name: "hello".to_owned(),
        }
    }
}
