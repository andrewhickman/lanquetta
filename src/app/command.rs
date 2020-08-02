use druid::{Selector, SingleUse};

use crate::grpc;

/// Begin sending the request
pub const START_SEND: Selector = Selector::new("app.start-send");

/// Finish sending the request
pub const FINISH_SEND: Selector<SingleUse<grpc::ResponseResult>> = Selector::new("app.finish-send");
