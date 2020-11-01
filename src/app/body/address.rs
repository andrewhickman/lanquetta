use std::string::ToString;
use std::{str::FromStr, sync::Arc};

use druid::{
    widget::{prelude::*, Button, Controller, Flex, Spinner, TextBox, ViewSwitcher},
    Data, Env, EventCtx, Lens, Target, Widget, WidgetExt as _,
};
use http::Uri;
use once_cell::sync::Lazy;

use crate::{
    app::{body::RequestState, command, theme},
    widget::{Empty, FormField, Icon, ValidationState},
};

#[derive(Debug, Clone, Data, Lens)]
pub(in crate::app) struct AddressState {
    #[lens(name = "uri_lens")]
    uri: ValidationState<String, Uri, String>,
    #[lens(name = "request_state_lens")]
    request_state: RequestState,
}

#[derive(Debug, Clone, Data, Lens)]
pub(in crate::app) struct State {
    address: AddressState,
    #[lens(name = "valid_lens")]
    can_send: bool,
}

struct AddressController {
    body_id: WidgetId,
}

pub(in crate::app) fn build(body_id: WidgetId) -> Box<dyn Widget<State>> {
    let address_form_field = FormField::new(theme::text_box_scope(
        TextBox::new()
            .with_placeholder("http://localhost:80")
            .expand_width(),
    ))
    .controller(AddressController { body_id });

    let spinner = ViewSwitcher::new(
        |&request_state: &RequestState, _| request_state,
        |&request_state, _, _| {
            let width = 24.0;
            let padding = (0.0, 0.0, theme::GUTTER_SIZE, 0.0);
            match request_state {
                RequestState::NotStarted => Empty.boxed(),
                RequestState::ConnectInProgress | RequestState::Active => {
                    Spinner::new().fix_width(width).padding(padding).boxed()
                }
                RequestState::Connected => Icon::check()
                    .with_color(theme::color::BOLD_ACCENT)
                    .fix_width(width)
                    .padding(padding)
                    .boxed(),
                RequestState::ConnectFailed => Icon::close()
                    .with_color(theme::color::ERROR)
                    .fix_width(width)
                    .padding(padding)
                    .boxed(),
            }
        },
    );

    let send_button = theme::button_scope(Button::new("Send").on_click(
        |ctx: &mut EventCtx, &mut valid: &mut bool, _: &Env| {
            if valid {
                ctx.submit_command(command::START_SEND.to(Target::Global));
            }
        },
    ))
    .env_scope(|env, &valid: &bool| {
        env.set(theme::DISABLED, !valid);
    });

    Flex::row()
        .with_flex_child(
            address_form_field
                .lens(AddressState::uri_lens)
                .lens(State::address),
            1.0,
        )
        .with_spacer(theme::GUTTER_SIZE)
        .with_child(
            spinner
                .lens(AddressState::request_state_lens)
                .lens(State::address),
        )
        .with_child(send_button.lens(State::valid_lens))
        .boxed()
}

static VALIDATE_URI: Lazy<Arc<dyn Fn(&str) -> Result<Uri, String> + Sync + Send>> =
    Lazy::new(|| Arc::new(|s| validate_uri(s)));

fn validate_uri(s: &str) -> Result<Uri, String> {
    let uri = Uri::from_str(s).map_err(|err| err.to_string())?;
    if uri.scheme().is_none() {
        return Err("URI must have scheme".to_owned());
    }
    Ok(uri)
}

impl State {
    pub fn new(address: AddressState, can_send: bool) -> Self {
        State { address, can_send }
    }

    pub fn address_state(&self) -> &AddressState {
        &self.address
    }

    pub fn into_address_state(self) -> AddressState {
        self.address
    }

    pub fn can_send(&self) -> bool {
        self.can_send
    }
}

impl Default for AddressState {
    fn default() -> Self {
        AddressState::new(String::new())
    }
}

impl AddressState {
    pub fn new(address: String) -> Self {
        AddressState {
            uri: ValidationState::new(address, VALIDATE_URI.clone()),
            request_state: RequestState::NotStarted,
        }
    }

    pub fn is_valid(&self) -> bool {
        self.uri.is_valid()
    }

    pub fn text(&self) -> &str {
        self.uri.text()
    }

    pub fn uri(&self) -> Option<&Uri> {
        self.uri.result().ok()
    }

    pub fn request_state(&self) -> RequestState {
        self.request_state
    }

    pub fn set_request_state(&mut self, request_state: RequestState) {
        self.request_state = request_state;
    }
}

impl<W> Controller<ValidationState<String, Uri, String>, W> for AddressController
where
    W: Widget<ValidationState<String, Uri, String>>,
{
    fn lifecycle(
        &mut self,
        child: &mut W,
        ctx: &mut LifeCycleCtx,
        event: &LifeCycle,
        data: &ValidationState<String, Uri, String>,
        env: &Env,
    ) {
        if let LifeCycle::WidgetAdded | LifeCycle::FocusChanged(false) = event {
            if let Ok(uri) = data.result() {
                ctx.submit_command(command::START_CONNECT.with(uri.clone()).to(self.body_id));
            }
        }

        child.lifecycle(ctx, event, data, env)
    }
}
