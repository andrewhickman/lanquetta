use druid::{Selector, SingleUse};

use crate::app::body::TabId;
use crate::grpc;
use crate::protobuf::ProtobufMethod;

/// Set the request method
pub const SELECT_METHOD: Selector<ProtobufMethod> = Selector::new("app.select-method");

/// Close a tab
pub const CLOSE_TAB: Selector<TabId> = Selector::new("app.close-tab");

/// Format a text box
pub const FORMAT: Selector = Selector::new("app.format");

/// Begin sending the request
pub const START_SEND: Selector = Selector::new("app.start-send");

/// Finish sending the request
pub const FINISH_SEND: Selector<SingleUse<grpc::ResponseResult>> = Selector::new("app.finish-send");
