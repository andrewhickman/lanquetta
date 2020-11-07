use std::sync::Arc;

use druid::widget::TextBox;
use druid::{Data, Lens, Widget, WidgetExt as _};

use crate::json::JsonText;
use crate::{grpc, protobuf, theme};

#[derive(Debug, Default, Clone, Data, Lens)]
pub(in crate::app) struct State {
    body: JsonText,
}

pub(in crate::app) fn build() -> impl Widget<State> {
    theme::text_box_scope(TextBox::multiline().with_font(theme::EDITOR_FONT)).lens(State::body)
}

impl State {
    pub fn new(result: grpc::ResponseResult) -> Self {
        let body = match result
            .and_then(|response| protobuf::to_json(&*response.body).map_err(Arc::new))
        {
            Ok(body) => JsonText::pretty(body),
            Err(err) => JsonText::plain_text(format!("{:?}", err)),
        };

        State { body }
    }
}