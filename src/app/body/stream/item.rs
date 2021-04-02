use std::sync::Arc;

use druid::{widget::TextBox, Application};
use druid::{Data, Lens, Widget, WidgetExt as _};
use serde::{Deserialize, Serialize};

use crate::json::{self, JsonText};
use crate::{grpc, protobuf, theme};

#[derive(Debug, Default, Clone, Data, Lens, Serialize, Deserialize)]
pub(in crate::app) struct State {
    #[serde(deserialize_with = "json::serde::deserialize_short")]
    body: JsonText,
}

pub(in crate::app) fn build() -> impl Widget<State> {
    theme::text_box_scope(TextBox::multiline().with_font(theme::EDITOR_FONT)).lens(State::body)
}

impl State {
    pub fn from_request(request: &grpc::Request) -> Self {
        let body = match protobuf::to_json(&*request.body) {
            Ok(body) => JsonText::short(body),
            Err(err) => JsonText::plain_text(format!("{:?}", err)),
        };

        State { body }
    }

    pub fn from_response(result: &grpc::ResponseResult) -> Self {
        let body = match result
            .as_ref()
            .map_err(|err| err.clone())
            .and_then(|response| protobuf::to_json(&*response.body).map_err(Arc::new))
        {
            Ok(body) => JsonText::short(body),
            Err(err) => JsonText::plain_text(format!("{:?}", err)),
        };

        State { body }
    }

    pub fn set_clipboard(&self) {
        Application::global()
            .clipboard()
            .put_string(self.body.original_data());
    }
}
