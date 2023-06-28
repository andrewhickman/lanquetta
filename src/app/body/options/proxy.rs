use std::sync::Arc;

use druid::{
    widget::{Checkbox, CrossAxisAlignment, Flex, Label, ViewSwitcher},
    ArcStr, Data, Env, EventCtx, Insets, Lens, Widget, WidgetExt, WidgetId,
};
use http::Uri;
use once_cell::sync::Lazy;

use crate::{
    error::fmt_err,
    lens,
    proxy::{Proxy, ProxyKind},
    theme::{self, font::HEADER_TWO},
    widget::{
        error_label, expander, input, readonly_input, ExpanderData, FinishEditController,
        FormField, Icon, ValidationFn, ValidationState, REFRESH,
    },
};

#[derive(Clone, Debug, Data, Lens)]
pub struct State {
    expanded: bool,
    input: ProxyValidationState,
    #[data(ignore)]
    #[lens(ignore)]
    target: Option<Uri>,
}

#[derive(Clone, Debug, Data, Lens)]
struct ProxyInput {
    uri: String,
    verify_certs: bool,
    system: bool,
    auth: String,
}

type ProxyValidationState = ValidationState<ProxyInput, Proxy, ArcStr>;

pub fn build() -> impl Widget<State> {
    let form_field = WidgetId::next();

    let textbox = ViewSwitcher::new(
        |state: &ProxyInput, _: &Env| state.system,
        move |&system: &bool, _: &ProxyInput, _: &Env| {
            if system {
                Flex::row()
                    .with_flex_child(readonly_input().lens(ProxyInput::uri), 1.0)
                    .with_spacer(theme::BODY_SPACER)
                    .with_child(
                        Icon::refresh()
                            .button(|ctx: &mut EventCtx, _, _| ctx.submit_command(REFRESH)),
                    )
                    .boxed()
            } else {
                input("http://localhost:80")
                    .controller(FinishEditController::new(form_field))
                    .lens(ProxyInput::uri)
                    .boxed()
            }
        },
    );

    let error = error_label(Insets::ZERO)
        .expand_width()
        .lens(lens::Project::new(|data: &ProxyValidationState| {
            data.display_error()
        }));

    expander::new(
        Label::new("Proxy settings").with_font(HEADER_TWO),
        FormField::new(
            form_field,
            Flex::column()
                .with_spacer(theme::BODY_SPACER)
                .with_child(
                    theme::check_box_scope(Checkbox::new("Use system proxy"))
                        .lens(ProxyInput::system),
                )
                .with_spacer(theme::BODY_SPACER)
                .with_child(textbox)
                // .with_child(error)
                .with_spacer(theme::BODY_SPACER)
                .with_child(
                    theme::check_box_scope(Checkbox::new(
                        "Enable certificate verification for proxy",
                    ))
                    .lens(ProxyInput::verify_certs),
                )
                .with_spacer(theme::BODY_SPACER)
                .with_child(Label::new("Proxy authorization").with_font(theme::font::HEADER_TWO))
                .with_spacer(theme::BODY_SPACER)
                .with_child(input("Basic YWxhZGRpbjpvcGVuc2VzYW1l").lens(ProxyInput::auth))
                .cross_axis_alignment(CrossAxisAlignment::Start),
        )
        .lens(State::input),
    )
}

impl State {
    pub fn new(proxy: Proxy, target: Option<Uri>) -> State {
        todo!()
        // let verify_certs = proxy.verify_certs();
        // let auth = proxy.auth();

        // State {
        //     input: ProxyValidationState::new(ProxyInput { uri: (), verify_certs: (), system: (), auth: () }, VALIDATE_PROXY.clone()),
        //     target,
        //     expanded: false,
        // }
    }

    pub fn get(&self) -> Proxy {
        self.input.result().unwrap_or_else(|| Proxy::none())
    }

    pub fn set_target(&mut self, uri: Option<Uri>) {
        todo!()
    }

    fn refresh(&mut self) {
        match &mut self.input {
            ProxyInput::System { proxy, display } => {
                *proxy = Proxy::system().map_err(|err| fmt_err(&err));
                *display = system_display_uri(proxy.as_ref(), self.target.as_ref());
            }
            ProxyInput::Custom { .. } => (),
        }
    }
}

impl Default for State {
    fn default() -> Self {
        Self {
            input: ValidationState::new(ProxyInput::default(), VALIDATE_PROXY.clone()),
            target: None,
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

static VALIDATE_PROXY: Lazy<ValidationFn<ProxyInput, Proxy, ArcStr>> =
    Lazy::new(|| Arc::new(validate_proxy));

fn validate_proxy(s: &ProxyInput) -> Result<Proxy, ArcStr> {
    Ok(Proxy::none())
    // if s.is_empty() {
    //     return Ok(Proxy::none());
    // }

    // let uri = Uri::from_str(s).map_err(|err| err.to_string())?;
    // if uri.scheme().is_none() {
    //     return Err("URI must have scheme".into());
    // }
    // Ok(Proxy::custom(uri))
}
