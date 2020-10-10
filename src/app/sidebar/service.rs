use std::sync::Arc;

use druid::{ArcStr, Data, Lens, Widget, WidgetExt, FontDescriptor, FontFamily};
use druid::widget::{Label, LineBreaking};

use crate::theme;
use crate::app::sidebar::method;
use crate::protobuf::ProtobufService;

#[derive(Debug, Clone, Data, Lens)]
pub(in crate::app) struct State {
    name: ArcStr,
    methods: Arc<[method::State]>,
}

pub(in crate::app) fn build() -> Box<dyn Widget<State>> {
    let name = Label::raw()
        .with_font(FontDescriptor::new(FontFamily::SANS_SERIF))
        .with_text_size(18.0)
        .with_line_break_mode(LineBreaking::Clip)
        .padding(theme::GUTTER_SIZE / 2.0)
        .lens(State::name);
    name.boxed()
}

impl From<ProtobufService> for State {
    fn from(service: ProtobufService) -> Self {
        State {
            name: service.name().into(),
            methods: service.methods().map(method::State::from).collect(),
        }
    }
}
