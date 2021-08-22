use std::sync::Arc;

use anyhow::Error;
use druid::{widget::TextBox, Application};
use druid::{Data, Lens, Widget, WidgetExt as _};
use serde::{Deserialize, Serialize};

use crate::json::{self, JsonText};
use crate::theme;

#[derive(Debug, Default, Clone, Data, Lens, Serialize, Deserialize)]
pub(in crate::app) struct State {
    #[serde(deserialize_with = "json::serde::deserialize_short")]
    body: JsonText,
}

pub(in crate::app) fn build() -> impl Widget<State> {
    theme::text_box_scope(TextBox::multiline().with_font(theme::EDITOR_FONT)).lens(State::body)
}

impl State {
    pub fn from_request(json: JsonText) -> Self {
        State { body: json }
    }

    pub fn from_response(result: Result<JsonText, Arc<Error>>) -> Self {
        let body = result.unwrap_or_else(|err| JsonText::plain_text(format!("{:?}", err)));

        State { body }
    }

    pub fn set_clipboard(&self) {
        Application::global()
            .clipboard()
            .put_string(self.body.original_data());
    }
}
