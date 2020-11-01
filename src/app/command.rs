use druid::{Selector, SingleUse};
use http::Uri;

use crate::grpc;
use crate::protobuf::ProtobufMethod;

/// Select the tab with the given method, or create new one
pub const SELECT_OR_CREATE_TAB: Selector<ProtobufMethod> =
    Selector::new("app.select-or-create-tab");

/// Remove a service
pub const REMOVE_SERVICE: Selector<usize> = Selector::new("app.remove-service");

/// Create a new tab with the given method
pub const CREATE_TAB: Selector<ProtobufMethod> = Selector::new("app.create-tab");

/// Format a text box
pub const FORMAT: Selector = Selector::new("app.format");

/// Begin connecting to the server
pub const START_CONNECT: Selector<Uri> = Selector::new("app.start-connect");

/// Finish connecting to the server
pub const FINISH_CONNECT: Selector<SingleUse<grpc::ConnectResult>> =
    Selector::new("app.finish-connect");

/// Begin sending a request
pub const START_SEND: Selector = Selector::new("app.start-send");

/// Finish sending a request
pub const FINISH_SEND: Selector<SingleUse<grpc::ResponseResult>> = Selector::new("app.finish-send");
