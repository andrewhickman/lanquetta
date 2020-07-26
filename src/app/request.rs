use druid::{Data, Lens, Widget, WidgetExt};

use crate::widget::TextArea;

#[derive(Debug, Default, Clone, Data, Lens)]
pub(in crate::app) struct State {
    body: String,
}

pub(in crate::app) fn build() -> impl Widget<State> {
    TextArea::new().lens(State::body).expand().boxed()
}
