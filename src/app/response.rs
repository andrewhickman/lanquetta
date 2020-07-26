use druid::{Data, Lens, Widget, WidgetExt};

use crate::grpc;
use crate::widget::TextArea;

#[derive(Debug, Default, Clone, Data, Lens)]
pub(in crate::app) struct State {
    body: String,
}

pub(in crate::app) fn build() -> impl Widget<State> {
    TextArea::new().styled().lens(State::body).boxed()
}

impl State {
    pub(in crate::app) fn update(&mut self, _result: grpc::ResponseResult) {}
}
