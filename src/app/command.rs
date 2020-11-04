use druid::Selector;

use crate::protobuf::ProtobufMethod;

/// Select the tab with the given method, or create new one
pub const SELECT_OR_CREATE_TAB: Selector<ProtobufMethod> =
    Selector::new("app.select-or-create-tab");

/// Remove a service
pub const REMOVE_SERVICE: Selector<usize> = Selector::new("app.remove-service");

/// Create a new tab with the given method
pub const CREATE_TAB: Selector<ProtobufMethod> = Selector::new("app.create-tab");

/// Begin connecting to the server
pub const CONNECT: Selector = Selector::new("app.connect");

/// Begin sending a request
pub const SEND: Selector = Selector::new("app.send");
