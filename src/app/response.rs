use druid::text::RichText;
use druid::widget::TextBox;
use druid::{Data, Lens, Widget, WidgetExt};

use crate::grpc;

#[derive(Debug, Default, Clone, Data, Lens)]
pub(in crate::app) struct State {
    body: String,
}

pub(in crate::app) fn build() -> Box<dyn Widget<State>> {
    TextBox::multiline().expand().lens(State::body).boxed()
}

impl State {
    pub(in crate::app) fn update(&mut self, result: grpc::ResponseResult) {
        match result {
            Ok(response) => self.body = todo!(),
            Err(err) => self.body = format!("{:?}", err),
        }
    }
}
