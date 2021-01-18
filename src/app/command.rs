use druid::Selector;

use crate::protobuf::ProtobufMethod;

/// Open the source code in a browser
pub const OPEN_GITHUB: Selector = Selector::new("app.open-github");

/// Select the tab with the given method, or create new one
pub const SELECT_OR_CREATE_TAB: Selector<ProtobufMethod> =
    Selector::new("app.select-or-create-tab");

/// Remove a service
pub const REMOVE_SERVICE: Selector<usize> = Selector::new("app.remove-service");

/// Create a new tab with the given method
pub const CREATE_TAB: Selector<ProtobufMethod> = Selector::new("app.create-tab");

/// Close the selected tab
pub const CLOSE_SELECTED_TAB: Selector = Selector::new("app.close-tab");

/// Close the selected tab
pub const SELECT_NEXT_TAB: Selector = Selector::new("app.select-next-tab");

/// Close the selected tab
pub const SELECT_PREV_TAB: Selector = Selector::new("app.select-prev-tab");

/// Begin connecting to the server
pub const CONNECT: Selector = Selector::new("app.connect");

/// Begin sending a request
pub const SEND: Selector = Selector::new("app.send");
