use druid::widget::{CrossAxisAlignment, Flex, Label, LineBreaking, List};
use druid::{ArcStr, Data, FontDescriptor, FontFamily, Lens, Widget, WidgetExt};

use crate::app::sidebar::method;
use crate::protobuf::ProtobufService;
use crate::theme;

#[derive(Debug, Clone, Data, Lens)]
pub(in crate::app) struct State {
    name: ArcStr,
    methods: im::Vector<method::State>,
}

pub(in crate::app) fn build() -> Box<dyn Widget<State>> {
    let name = Label::raw()
        .with_font(FontDescriptor::new(FontFamily::SANS_SERIF))
        .with_text_size(18.0)
        .with_line_break_mode(LineBreaking::Clip)
        .padding(theme::GUTTER_SIZE / 2.0)
        .lens(State::name);
    let methods = List::new(method::build).lens(State::methods);

    Flex::column()
        .cross_axis_alignment(CrossAxisAlignment::Start)
        .with_child(name)
        .with_child(methods)
        .boxed()
}

impl From<ProtobufService> for State {
    fn from(service: ProtobufService) -> Self {
        State {
            name: service.name().into(),
            methods: service.methods().map(method::State::from).collect(),
        }
    }
}
