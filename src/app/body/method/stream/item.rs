use std::mem::{self, Discriminant};

use anyhow::Error;
use druid::{
    lens::Field,
    widget::{Label, LineBreaking, TextBox, ViewSwitcher},
    Application, Data, Env, Lens, Widget, WidgetExt as _,
};
use serde::{Deserialize, Serialize};
use tonic::metadata::MetadataMap;

use crate::{app::fmt_err, theme};
use crate::{
    app::metadata,
    json::{self, JsonText},
};

#[derive(Debug, Clone, Data, Serialize, Deserialize)]
pub(in crate::app) enum State {
    #[serde(deserialize_with = "json::serde::deserialize_short")]
    Payload(JsonText),
    Error(String),
    Metadata(metadata::State),
}

pub(in crate::app) fn build() -> impl Widget<State> {
    ViewSwitcher::new(
        |data: &State, _: &Env| mem::discriminant(data),
        |_: &Discriminant<State>, data: &State, _: &Env| match data {
            State::Payload(_) => theme::text_box_scope(
                TextBox::multiline()
                    .readonly()
                    .with_font(theme::EDITOR_FONT),
            )
            .lens(State::lens_payload())
            .boxed(),
            State::Error(_) => theme::error_label_scope(
                Label::dynamic(|data: &String, _: &Env| data.clone())
                    .with_line_break_mode(LineBreaking::WordWrap),
            )
            .lens(State::lens_error())
            .boxed(),
            State::Metadata(_) => metadata::build().lens(State::lens_metadata()).boxed(),
        },
    )
}

impl State {
    fn lens_payload() -> impl Lens<State, JsonText> {
        Field::new(
            |data| match data {
                State::Payload(payload) => payload,
                _ => panic!("unexpected variant"),
            },
            |data| match data {
                State::Payload(payload) => payload,
                _ => panic!("unexpected variant"),
            },
        )
    }

    fn lens_error() -> impl Lens<State, String> {
        Field::new(
            |data| match data {
                State::Error(err) => err,
                _ => panic!("unexpected variant"),
            },
            |data| match data {
                State::Error(err) => err,
                _ => panic!("unexpected variant"),
            },
        )
    }

    fn lens_metadata() -> impl Lens<State, metadata::State> {
        Field::new(
            |data| match data {
                State::Metadata(metadata) => metadata,
                _ => panic!("unexpected variant"),
            },
            |data| match data {
                State::Metadata(metadata) => metadata,
                _ => panic!("unexpected variant"),
            },
        )
    }

    pub fn from_request(json: JsonText) -> Self {
        State::Payload(json)
    }

    pub fn from_response(result: Result<JsonText, Error>) -> Self {
        match result {
            Ok(payload) => State::Payload(payload),
            Err(err) => State::Error(fmt_grpc_err(&err)),
        }
    }

    pub fn from_metadata(metadata: MetadataMap) -> State {
        State::Metadata(metadata::state_from_tonic(metadata))
    }

    pub fn can_copy(&self) -> bool {
        match self {
            State::Payload(_) | State::Error(_) => true,
            State::Metadata(_) => false,
        }
    }

    pub fn set_clipboard(&self) {
        let data = match self {
            State::Payload(payload) => payload.original_data(),
            State::Error(err) => err.as_str(),
            State::Metadata(_) => return,
        };

        Application::global().clipboard().put_string(data);
    }
}

fn fmt_grpc_err(err: &anyhow::Error) -> String {
    fmt_err(err)
}
