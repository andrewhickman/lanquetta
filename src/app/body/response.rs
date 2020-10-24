use druid::widget::TextBox;
use druid::{Data, Lens, Widget, WidgetExt as _};

use crate::json::JsonText;
use crate::{grpc, theme};

#[derive(Debug, Default, Clone, Data, Lens)]
pub(in crate::app) struct State {
    body: JsonText,
}

pub(in crate::app) fn build() -> Box<dyn Widget<State>> {
    TextBox::multiline()
        .with_font(theme::EDITOR_FONT)
        .expand()
        .lens(State::body)
        .boxed()
}

impl State {
    pub(in crate::app) fn update(&mut self, result: grpc::ResponseResult) {
        match result {
            Ok(_) => self.body = todo!(),
            Err(err) => self.body = JsonText::from(format!("{:?}", err)),
        }
    }
}
