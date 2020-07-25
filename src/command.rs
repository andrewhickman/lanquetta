use std::sync::Arc;

use druid::Selector;

use crate::address::Address;

pub const CONNECT: Selector<(Arc<Address>, bool)> = Selector::new("grpc-client.connect");
