use druid::lens::{self, Lens};
use druid::{ArcStr, Data, Widget};

use crate::protobuf::ProtobufMethod;

#[derive(Debug, Clone, Data)]
pub(in crate::app) struct State {
    method: ProtobufMethod,
}

pub(in crate::app) fn build() -> Box<dyn Widget<State>> {
    todo!()
}

impl State {
    fn name() -> impl Lens<State, ArcStr> {
        struct NameLens;

        impl Lens<State, ArcStr> for NameLens {
            fn with<V, F: FnOnce(&ArcStr) -> V>(&self, data: &State, f: F) -> V {
                f(data.method.name())
            }

            fn with_mut<V, F: FnOnce(&mut ArcStr) -> V>(&self, data: &mut State, f: F) -> V {
                f(&mut data.method.name().clone())
            }
        }

        NameLens
    }
}

impl From<ProtobufMethod> for State {
    fn from(method: ProtobufMethod) -> Self {
        State { method }
    }
}
