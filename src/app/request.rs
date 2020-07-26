use druid::{Data, Lens, Widget, WidgetExt};

use crate::widget::TextArea;

#[derive(Debug, Clone, Data, Lens)]
pub(in crate::app) struct State {
    body: String,
}

pub(in crate::app) fn build() -> impl Widget<State> {
    TextArea::new().styled().lens(State::body).boxed()
}

impl Default for State {
    fn default() -> Self {
        State {
            body: "hello\nworld\n1\n2\n3\n4\n5\n6\n7\n8".to_owned(),
        }
    }
}
