use druid::{Data, Lens};

use crate::address::AddressState;
use crate::connection::ConnectionState;

/// The top-level application state.
#[derive(Clone, Data, Lens)]
pub(crate) struct AppState {
    pub name: String,
    pub address: AddressState,
    pub connection: ConnectionState,
}

impl AppState {
    pub fn new() -> Self {
        AppState {
            name: "hello".to_owned(),
            address: AddressState::new(),
            connection: Default::default(),
        }
    }
}
