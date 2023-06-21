use std::mem::{self, Discriminant};

use anyhow::Error;
use druid::{
    lens::Field,
    widget::{CrossAxisAlignment, Flex, Label, LineBreaking, Maybe, TextBox, ViewSwitcher},
    Application, ArcStr, Data, Env, Lens, Widget, WidgetExt as _,
};
use prost_reflect::{DescriptorPool, DynamicMessage, Value};
use serde::{Deserialize, Serialize};
use tonic::{metadata::MetadataMap, Code, Status};

use crate::{
    app::body::fmt_connect_err,
    grpc,
    theme::{self, INVALID},
    widget::{code_area, Empty},
};
use crate::{
    app::metadata,
    json::{self, JsonText},
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
                .with_child(
                    theme::error_label_scope(
                        Label::dynamic(|data: &ArcStr, _: &Env| data.to_string())
                            .with_line_break_mode(LineBreaking::WordWrap),
                    )
                    .lens(ErrorDetail::message),
                )
                .with_child(
                    Maybe::new(
                        || {
                            code_area(false)
                                .env_scope(|env: &mut Env, _: &JsonText| env.set(INVALID, true))
                        },
                        || Empty,
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

fn fmt_grpc_err(err: &anyhow::Error) -> ArcStr {
    if let Some(status) = err.downcast_ref::<Status>() {
        if status.message().is_empty() {
            fmt_code(status.code()).into()
        } else {
            format!("{}: {}", fmt_code(status.code()), status.message()).into()
        }
    } else {
        fmt_connect_err(err)
    }
}

fn fmt_code(code: Code) -> &'static str {
    match code {
        Code::Ok => "OK",
        Code::Cancelled => "CANCELLED",
        Code::Unknown => "UNKNOWN",
        Code::InvalidArgument => "INVALID_ARGUMENT",
        Code::DeadlineExceeded => "DEADLINE_EXCEEDED",
        Code::NotFound => "NOT_FOUND",
        Code::AlreadyExists => "ALREADY_EXISTS",
        Code::PermissionDenied => "PERMISSION_DENIED",
        Code::ResourceExhausted => "RESOURCE_EXHAUSTED",
        Code::FailedPrecondition => "FAILED_PRECONDITION",
        Code::Aborted => "ABORTED",
        Code::OutOfRange => "OUT_OF_RANGE",
        Code::Unimplemented => "UNIMPLEMENTED",
        Code::Internal => "INTERNAL",
        Code::Unavailable => "UNAVAILABLE",
        Code::DataLoss => "DATA_LOSS",
        Code::Unauthenticated => "UNAUTHENTICATED",
    }
}
