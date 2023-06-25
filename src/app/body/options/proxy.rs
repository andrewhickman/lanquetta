use std::{str::FromStr, sync::Arc};

use druid::{
    lens::Field,
    widget::{Flex, ViewSwitcher, Checkbox},
    ArcStr, Data, Env, Insets, Lens, Widget, WidgetExt,
};
use http::Uri;
use once_cell::sync::Lazy;

use crate::{
    error::fmt_err,
    lens::{self, Project},
    proxy::Proxy,
    widget::{error_label, input, readonly_input, FormField, ValidationFn, ValidationState}, theme,
};

#[derive(Clone, Debug, Data, Lens)]
pub struct State {
    input: ProxyInput,
    #[data(ignore)]
    #[lens(ignore)]
    target: Option<Uri>,
}

#[derive(Clone, Debug, Data)]
enum ProxyInput {
    System(Result<Proxy, ArcStr>),
    Custom(ProxyValidationState),
}

type ProxyValidationState = ValidationState<String, Proxy, ArcStr>;

pub fn build() -> impl Widget<State> {
    let textbox = ViewSwitcher::new(
        |state: &State, _: &Env| state.is_system(),
        |&is_system: &bool, _: &State, _: &Env| {
            if is_system {
                readonly_input()
                    .lens(Project::new(|state: &State| state.system_display_url()))
                    .boxed()
            } else {
                FormField::text_box(input("http://localhost:80"))
                    .lens(State::custom_lens())
                    .boxed()
            }
        },
    );

    let error = error_label(Insets::ZERO)
        .expand_width()
        .lens(lens::Project::new(|data: &State| {
            data.input.display_error()
        }));

    Flex::column()
        .with_child(theme::check_box_scope(Checkbox::new("Use system proxy")).lens(State::system_toggle_lens()))
        .with_spacer(theme::BODY_SPACER)
        .with_child(textbox)
        .with_child(error)
}

impl State {
    pub fn new(options: Proxy, uri: Option<Uri>) -> State {
        todo!()
    }

    pub fn get(&self) -> Proxy {
        match &self.input {
            ProxyInput::System(system) => system.clone().unwrap_or_else(|_| Proxy::none()),
            ProxyInput::Custom(custom) => {
                custom.result().cloned().unwrap_or_else(|_| Proxy::none())
            }
        }
    }

    fn is_system(&self) -> bool {
        match &self.input {
            ProxyInput::System(_) => true,
            ProxyInput::Custom(_) => false,
        }
    }

    fn system_display_url(&self) -> String {
        let uri = match (&self.target, &self.input) {
            (Some(uri), ProxyInput::System(Ok(system))) => system.get_proxy(uri),
            (None, ProxyInput::System(Ok(system))) => system.get_default(),
            _ => return String::default(),
        };

        if let Some(uri) = uri {
            uri.to_string()
        } else {
            String::default()
        }
    }

    fn toggle_system(&mut self) {
        match &mut self.input {
            ProxyInput::System(system) => {
                self.input = ProxyInput::Custom(ProxyValidationState::new(
                    self.system_display_url(),
                    VALIDATE_PROXY.clone(),
                ));
            }
            ProxyInput::Custom(_) => {
                self.input = ProxyInput::System(Proxy::system().map_err(|err| fmt_err(&err)));
            }
        }
    }

    fn system_toggle_lens() -> impl Lens<State, bool> {
        struct SystemToggleLens;

        impl Lens<State, bool> for SystemToggleLens {
            fn with<V, F: FnOnce(&bool) -> V>(&self, data: &State, f: F) -> V {
                f(&data.is_system())
            }

            fn with_mut<V, F: FnOnce(&mut bool) -> V>(&self, data: &mut State, f: F) -> V {
                let mut is_system = data.is_system();
                let result = f(&mut is_system);
                if data.is_system() != is_system {
                    data.toggle_system();
                }

                result
            }
        }

        SystemToggleLens
    }

    fn custom_lens() -> impl Lens<State, ProxyValidationState> {
        Field::new(
            |data: &State| match &data.input {
                ProxyInput::Custom(custom) => custom,
                _ => panic!("unexpected variant"),
            },
            |data: &mut State| match &mut data.input {
                ProxyInput::Custom(custom) => custom,
                _ => panic!("unexpected variant"),
            },
        )
    }
}

impl ProxyInput {
    fn display_error(&self) -> Option<ArcStr> {
        match self {
            ProxyInput::System(result) => result.as_ref().err().cloned(),
            ProxyInput::Custom(result) => result.error(),
        }
    }
}

impl Default for State {
    fn default() -> Self {
        Self {
            input: ProxyInput::Custom(ValidationState::new(
                String::default(),
                VALIDATE_PROXY.clone(),
            )),
            target: None,
        }
    }
}

static VALIDATE_PROXY: Lazy<ValidationFn<String, Proxy, ArcStr>> =
    Lazy::new(|| Arc::new(validate_proxy));

#[allow(clippy::ptr_arg)]
fn validate_proxy(s: &String) -> Result<Proxy, ArcStr> {
    if s.is_empty() {
        return Ok(Proxy::none());
    }

    let uri = Uri::from_str(s).map_err(|err| err.to_string())?;
    if uri.scheme().is_none() {
        return Err("URI must have scheme".into());
    }
    Ok(Proxy::custom(uri))
}
