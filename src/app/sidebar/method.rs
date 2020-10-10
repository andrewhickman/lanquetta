use druid::{
    widget::{Label, LineBreaking},
    TextAlignment,
};
use druid::{ArcStr, Data, FontDescriptor, FontFamily, Lens, Widget, WidgetExt};

use crate::protobuf::ProtobufMethod;
use crate::theme;

#[derive(Debug, Clone, Data)]
pub(in crate::app) struct State {
    method: ProtobufMethod,
}

pub(in crate::app) fn build() -> Box<dyn Widget<State>> {
    Label::raw()
        .with_font(FontDescriptor::new(FontFamily::SANS_SERIF))
        .with_text_size(16.0)
        .with_line_break_mode(LineBreaking::Clip)
        .padding(theme::GUTTER_SIZE / 4.0)
        .lens(State::name())
        .boxed()
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
