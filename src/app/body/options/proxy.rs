use std::{str::FromStr, sync::Arc};

use druid::{
    lens::Field,
    widget::{Checkbox, Flex, ViewSwitcher},
    ArcStr, Data, Env, Insets, Lens, Widget, WidgetExt,
};
use http::Uri;
use once_cell::sync::Lazy;

use crate::{
    error::fmt_err,
    lens::{self, Project},
    proxy::{Proxy, ProxyKind},
    theme,
    widget::{error_label, input, readonly_input, FormField, ValidationFn, ValidationState},
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
    System {
        proxy: Result<Proxy, ArcStr>,
        display: Arc<String>,
    },
    Custom {
        uri: ProxyValidationState,
    },
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
        .lens(lens::Project::new(|data: &State| data.display_error()));

    Flex::column()
        .with_child(
            theme::check_box_scope(Checkbox::new("Use system proxy"))
                .lens(State::system_toggle_lens()),
        )
        .with_spacer(theme::BODY_SPACER)
        .with_child(textbox)
        .with_child(error)
}

impl State {
    pub fn new(proxy: Proxy, target: Option<Uri>) -> State {
        let input = match proxy.kind() {
            ProxyKind::None => ProxyInput::Custom {
                uri: ValidationState::new(String::default(), VALIDATE_PROXY.clone()),
            },
            ProxyKind::System => ProxyInput::System {
                display: system_display_uri(Ok(&proxy), target.as_ref()),
                proxy: Ok(proxy),
            },
            ProxyKind::Custom(uri) => ProxyInput::Custom {
                uri: ValidationState::new(uri.to_string(), VALIDATE_PROXY.clone()),
            },
        };

        State { input, target }
    }

    pub fn get(&self) -> Proxy {
        match &self.input {
            ProxyInput::System { proxy, .. } => proxy.clone().unwrap_or_else(|_| Proxy::none()),
            ProxyInput::Custom { uri } => uri.result().cloned().unwrap_or_else(|_| Proxy::none()),
        }
    }

    pub fn set_target(&mut self, uri: Option<Uri>) {
        self.target = uri;
        if let ProxyInput::System { proxy, display } = &mut self.input {
            *display = system_display_uri(proxy.as_ref(), self.target.as_ref());
        }
    }

    fn is_system(&self) -> bool {
        match &self.input {
            ProxyInput::System { .. } => true,
            ProxyInput::Custom { .. } => false,
        }
    }

    fn system_display_url(&self) -> Arc<String> {
        match &self.input {
            ProxyInput::System { display, .. } => display.clone(),
            ProxyInput::Custom { .. } => panic!("unexpected variant"),
        }
    }

    fn display_error(&self) -> Option<ArcStr> {
        match &self.input {
            ProxyInput::System { proxy, .. } => proxy.as_ref().err().cloned(),
            ProxyInput::Custom { uri } => uri.display_error(),
        }
    }

    fn toggle_system(&mut self) {
        match &mut self.input {
            ProxyInput::System { display, .. } => {
                self.input = ProxyInput::Custom {
                    uri: ProxyValidationState::new((**display).clone(), VALIDATE_PROXY.clone()),
                };
            }
            ProxyInput::Custom { .. } => {
                let proxy = Proxy::system().map_err(|err| fmt_err(&err));
                let display = system_display_uri(proxy.as_ref(), self.target.as_ref());

                self.input = ProxyInput::System { proxy, display };
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
                ProxyInput::Custom { uri } => uri,
                _ => panic!("unexpected variant"),
            },
            |data: &mut State| match &mut data.input {
                ProxyInput::Custom { uri } => uri,
                _ => panic!("unexpected variant"),
            },
        )
    }
}

impl Default for State {
    fn default() -> Self {
        Self {
            input: ProxyInput::Custom {
                uri: ValidationState::new(String::default(), VALIDATE_PROXY.clone()),
            },
            target: None,
        }
    }
}

fn system_display_uri(proxy: Result<&Proxy, &ArcStr>, target: Option<&Uri>) -> Arc<String> {
    let uri = match (proxy, target) {
        (Ok(proxy), Some(target)) => proxy.get_proxy(target),
        (Ok(proxy), None) => proxy.get_default(),
        _ => return Arc::default(),
    };

    match uri {
        Some(uri) => Arc::new(uri.to_string()),
        None => Arc::default(),
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
