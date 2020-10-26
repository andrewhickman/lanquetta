use druid::{Selector, SingleUse};

use crate::grpc;
use crate::protobuf::ProtobufMethod;

/// Select the tab with the given method, or create new one
pub const SELECT_OR_CREATE_TAB: Selector<ProtobufMethod> =
    Selector::new("app.select-or-create-tab");

/// Create a new tab with the given method
pub const CREATE_TAB: Selector<ProtobufMethod> = Selector::new("app.create-tab");

/// Format a text box
pub const FORMAT: Selector = Selector::new("app.format");

/// Begin sending a request
pub const START_SEND: Selector = Selector::new("app.start-send");

/// Finish sending a request
pub const FINISH_SEND: Selector<SingleUse<grpc::ResponseResult>> = Selector::new("app.finish-send");
