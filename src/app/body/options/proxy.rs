use std::sync::Arc;

use druid::{
    widget::{prelude::*, Checkbox, Controller, CrossAxisAlignment, Flex, Label, ViewSwitcher},
    ArcStr, Data, EventCtx, Insets, Lens, WidgetExt, WidgetId,
};
use http::Uri;
use once_cell::sync::Lazy;

use crate::{
    proxy::Proxy,
    theme::{self, font::HEADER_TWO},
    widget::{
        env_error_label, expander, input, readonly_input, ExpanderData, FinishEditController,
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

#[derive(Default, Clone, Debug, Data, Lens)]
struct ProxyInput {
    uri: String,
    verify_certs: bool,
    system: bool,
    auth: String,
}

type ProxyValidationState = ValidationState<ProxyInput, Proxy>;

struct ProxyController;

pub fn build() -> impl Widget<State> {
    let form_field = WidgetId::next();

    let textbox = ViewSwitcher::new(
        |data: &ProxyInput, _: &Env| data.system,
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
                .with_child(env_error_label(Insets::ZERO))
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
        self.input
            .result()
            .cloned()
            .unwrap_or_else(|_| Proxy::none())
    }

    pub fn set_target(&mut self, uri: Option<Uri>) {
        self.target = uri;

        if self.input.text().system {
            let display = self.system_display_uri();
            self.input.with_text_mut(|i| {
                i.uri = display;
            })
        }
    }

    fn update_system_display_uri(&mut self) {
        debug_assert!(self.input.text().system);
        let display = self.system_display_uri();
        self.input.with_text_mut(|i| {
            i.uri = display;
        })
    }

    fn system_display_uri(&self) -> String {
        todo!()
    }
}

impl Default for State {
    fn default() -> Self {
        Self {
            input: ValidationState::new(ProxyInput::default(), VALIDATE_PROXY.clone()),
            expanded: false,
            target: None,
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

impl<W> Controller<State, W> for ProxyController
where
    W: Widget<State>,
{
    fn event(
        &mut self,
        child: &mut W,
        ctx: &mut EventCtx,
        event: &Event,
        data: &mut State,
        env: &Env,
    ) {
        let proxy = data.input.result().ok().cloned();

        child.event(ctx, event, data, env);

        if data.input.text().system && !data.input.result().ok().cloned().same(&proxy) {
            data.update_system_display_uri();
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

static VALIDATE_PROXY: Lazy<ValidationFn<ProxyInput, Proxy>> =
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
