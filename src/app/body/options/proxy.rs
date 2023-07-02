use std::{str::FromStr, sync::Arc};

use druid::{
    widget::{prelude::*, Checkbox, CrossAxisAlignment, Flex, Label},
    ArcStr, Data, Insets, Lens, WidgetExt, WidgetId,
};
use http::Uri;
use once_cell::sync::Lazy;

use crate::{
    proxy::Proxy,
    theme::{self, font::HEADER_TWO},
    widget::{
        env_error_label, expander, input, ExpanderData, FinishEditController, FormField,
        ValidationFn, ValidationState,
    },
};

#[derive(Clone, Debug, Data, Lens)]
pub struct State {
    expanded: bool,
    input: ProxyValidationState,
}

#[derive(Clone, Debug, Data, Lens)]
struct ProxyInput {
    uri: String,
    verify_certs: bool,
}

type ProxyValidationState = ValidationState<ProxyInput, Proxy>;

pub fn build() -> impl Widget<State> {
    let form_field = WidgetId::next();

    expander::new(
        Label::new("Proxy settings").with_font(HEADER_TWO),
        FormField::new(
            form_field,
            Flex::column()
                .with_spacer(theme::BODY_SPACER)
                .with_child(Label::new("Proxy url").with_font(theme::font::HEADER_TWO))
                .with_spacer(theme::BODY_SPACER)
                .with_child(
                    input("http://localhost:80")
                        .controller(FinishEditController::new(form_field))
                        .lens(ProxyInput::uri),
                )
                .with_child(env_error_label(Insets::ZERO).expand_width())
                .with_spacer(theme::BODY_SPACER)
                .with_child(
                    theme::check_box_scope(Checkbox::new(
                        "Enable certificate verification for proxy",
                    ))
                    .lens(ProxyInput::verify_certs),
                )
                .cross_axis_alignment(CrossAxisAlignment::Start),
        )
        .lens(State::input),
    )
}

impl State {
    pub fn new(proxy: Proxy) -> State {
        match proxy {
            Proxy::None => State::default(),
            Proxy::Custom { uri, verify_certs } => State {
                expanded: true,
                input: ValidationState::new(
                    ProxyInput {
                        uri: uri.to_string(),
                        verify_certs,
                    },
                    VALIDATE_PROXY.clone(),
                ),
            },
        }
    }

    pub fn get(&self) -> Proxy {
        self.input.result().cloned().unwrap_or(Proxy::None)
    }
}

impl Default for State {
    fn default() -> Self {
        Self {
            input: ValidationState::new(
                ProxyInput {
                    uri: String::new(),
                    verify_certs: true,
                },
                VALIDATE_PROXY.clone(),
            ),
            expanded: false,
        }
    }
}

impl ExpanderData for State {
    fn expanded(&self, _: &Env) -> bool {
        self.expanded
    }

    fn toggle_expanded(&mut self, _: &Env) {
        self.expanded = !self.expanded
    }
}

static VALIDATE_PROXY: Lazy<ValidationFn<ProxyInput, Proxy>> =
    Lazy::new(|| Arc::new(validate_proxy));

fn validate_proxy(s: &ProxyInput) -> Result<Proxy, ArcStr> {
    if s.uri.is_empty() {
        return Ok(Proxy::None);
    }

    let uri = Uri::from_str(&s.uri).map_err(|err| err.to_string())?;
    if uri.scheme().is_none() {
        return Err("URI must have scheme".into());
    }
    Ok(Proxy::Custom {
        uri,
        verify_certs: s.verify_certs,
    })
}
