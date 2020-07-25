use druid::widget::TextBox;
use druid::{Data, Lens, Widget, WidgetExt};

#[derive(Debug, Default, Clone, Data, Lens)]
pub(in crate::app) struct State {
    body: String,
}

pub(in crate::app) fn build() -> impl Widget<State> {
    TextBox::new().lens(State::body).expand().boxed()
}
