use std::mem::{self, Discriminant};

use anyhow::Error;
use druid::{
    lens::Field,
    widget::{CrossAxisAlignment, Flex, Maybe, ViewSwitcher},
    Application, ArcStr, Data, Env, Insets, Lens, Widget, WidgetExt as _,
};
use prost_reflect::{DescriptorPool, DynamicMessage, Value};
use serde::{Deserialize, Serialize};
use tonic::{metadata::MetadataMap, Status};

use crate::{
    app::metadata,
    error::fmt_grpc_err,
    grpc,
    json::{self, JsonText},
    lens,
    theme::INVALID,
    widget::{code_area, empty, error_label},
};

#[derive(Debug, Clone, Data, Serialize, Deserialize)]
pub(in crate::app) enum State {
    #[serde(deserialize_with = "json::serde::deserialize_short")]
    Payload(JsonText),
    Error(ErrorDetail),
    Metadata(metadata::State),
}

#[derive(Debug, Clone, Lens, Data, Serialize, Deserialize)]
pub struct ErrorDetail {
    message: ArcStr,
    #[serde(deserialize_with = "json::serde::deserialize_short_opt")]
    details: Option<JsonText>,
}

pub(in crate::app) fn build() -> impl Widget<State> {
    ViewSwitcher::new(
        |data: &State, _: &Env| mem::discriminant(data),
        |_: &Discriminant<State>, data: &State, _: &Env| match data {
            State::Payload(_) => code_area(false).lens(State::payload_lens()).boxed(),
            State::Error(_) => Flex::column()
                .cross_axis_alignment(CrossAxisAlignment::Fill)
                .with_child(error_label(Insets::ZERO).lens(lens::Project::new(
                    |data: &ErrorDetail| Some(data.message.clone()),
                )))
                .with_child(
                    Maybe::new(
                        || {
                            code_area(false)
                                .env_scope(|env: &mut Env, _: &JsonText| env.set(INVALID, true))
                        },
                        empty,
                    )
                    .lens(ErrorDetail::details),
                )
                .lens(State::error_lens())
                .boxed(),
            State::Metadata(_) => metadata::build().lens(State::metadata_lens()).boxed(),
        },
    )
}

impl State {
    fn payload_lens() -> impl Lens<State, JsonText> {
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

    fn error_lens() -> impl Lens<State, ErrorDetail> {
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

    fn metadata_lens() -> impl Lens<State, metadata::State> {
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

    pub fn from_response(pool: &DescriptorPool, result: Result<JsonText, Error>) -> Self {
        match result {
            Ok(payload) => State::Payload(payload),
            Err(err) => {
                let message = fmt_grpc_err(&err);
                let details = error_details(pool, &err).map(|payload| {
                    let response = grpc::Response::new(payload);
                    JsonText::short(response.to_json())
                });

                State::Error(ErrorDetail { message, details })
            }
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
            State::Error(err) => {
                if let Some(detail) = &err.details {
                    detail.original_data()
                } else {
                    err.message.as_ref()
                }
            }
            State::Metadata(_) => return,
        };

        Application::global().clipboard().put_string(data);
    }
}

fn error_details(pool: &DescriptorPool, err: &anyhow::Error) -> Option<DynamicMessage> {
    let Some(status) = err.downcast_ref::<Status>() else {
        return None
    };

    if status.details().is_empty() {
        return None;
    };

    let Some(desc) = pool.get_message_by_name("google.rpc.Status") else {
        return None
    };

    let Ok(mut payload) = DynamicMessage::decode(desc, status.details()) else {
        return None
    };

    for detail in payload.get_field_by_name_mut("details")?.as_list_mut()? {
        let Some(message) = detail.as_message_mut() else { return None };

        let type_url = message.get_field_by_name("type_url")?.as_str()?.to_owned();
        if pool
            .get_message_by_name(type_url.strip_prefix("type.googleapis.com/")?)
            .is_none()
        {
            let value = message.get_field_by_name("value")?.as_bytes()?.clone();

            let mut unknown_message =
                DynamicMessage::new(pool.get_message_by_name("lanquetta.UnknownAny")?);
            unknown_message
                .try_set_field_by_name("type_url", Value::String(type_url))
                .ok()?;
            unknown_message
                .try_set_field_by_name("value", Value::Bytes(value))
                .ok()?;
            *message = unknown_message;
        }
    }

    Some(payload)
}
