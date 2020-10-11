use druid::{Selector, SingleUse};

use crate::grpc;
use crate::protobuf::ProtobufMethod;

/// Set the request method
pub const SELECT_METHOD: Selector<ProtobufMethod> = Selector::new("app.select-method");

/// Begin sending the request
pub const START_SEND: Selector = Selector::new("app.start-send");

/// Finish sending the request
pub const FINISH_SEND: Selector<SingleUse<grpc::ResponseResult>> = Selector::new("app.finish-send");
