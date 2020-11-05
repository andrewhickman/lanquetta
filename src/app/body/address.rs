use std::string::ToString;
use std::{str::FromStr, sync::Arc};

use druid::{
    widget::{
        prelude::*, Button, Controller, CrossAxisAlignment, Either, Flex, Label, Spinner, TextBox,
        ViewSwitcher,
    },
    Data, Env, EventCtx, Lens, Target, Widget, WidgetExt as _,
};
use http::Uri;
use once_cell::sync::Lazy;

use crate::{
    app::{body::RequestState, command, theme},
    widget::{Empty, FormField, Icon, ValidationState, FINISH_EDIT},
};

type AddressValidationState = ValidationState<String, Uri, String>;

#[derive(Debug, Clone, Data, Lens)]
pub(in crate::app) struct AddressState {
    #[lens(name = "uri_lens")]
    uri: AddressValidationState,
    #[lens(name = "request_state_lens")]
    request_state: RequestState,
}

#[derive(Debug, Clone, Data, Lens)]
pub(in crate::app) struct State {
    address: AddressState,
    #[lens(name = "can_send_lens")]
    can_send: bool,
}

struct AddressController {
    body_id: WidgetId,
}

pub(in crate::app) fn build(body_id: WidgetId) -> Box<dyn Widget<State>> {
    let address_textbox = FormField::new(theme::text_box_scope(
        TextBox::new()
            .with_placeholder("http://localhost:80")
            .expand_width(),
    ))
    .controller(AddressController { body_id })
    .lens(AddressState::uri_lens);

    let error_label =
        theme::error_label_scope(Label::dynamic(|data: &AddressValidationState, _| {
            data.result().err().cloned().unwrap_or_default()
        }));
    let error = Either::new(
        |data: &AddressValidationState, _| !data.is_pristine_or_valid(),
        error_label,
        Empty,
    )
    .expand_width();

    let address_form_field = Flex::column()
        .with_child(address_textbox)
        .with_child(error.lens(AddressState::uri_lens));

    let spinner = ViewSwitcher::new(
        |&request_state: &RequestState, _| request_state,
        |&request_state, _, _| match request_state {
            RequestState::NotStarted => Empty.boxed(),
            RequestState::ConnectInProgress | RequestState::Active => {
                layout_spinner(Spinner::new(), 2.0)
            }
            RequestState::Connected => {
                layout_spinner(Icon::check().with_color(theme::color::BOLD_ACCENT), 0.0)
            }
            RequestState::ConnectFailed => {
                layout_spinner(Icon::close().with_color(theme::color::ERROR), 0.0)
            }
        },
    );

    let send_button = theme::button_scope(
        Button::dynamic(|data: &State, _| match data.address.request_state {
            RequestState::Active => "Sending...".to_owned(),
            _ => "Send".to_owned(),
        })
        .on_click(|ctx: &mut EventCtx, data: &mut State, _: &Env| {
            if data.can_send {
                ctx.submit_command(command::SEND.to(Target::Global));
            }
        }),
    )
    .env_scope(|env, data: &State| {
        env.set(theme::DISABLED, !data.can_send);
    });

    Flex::row()
        .cross_axis_alignment(CrossAxisAlignment::Start)
        .with_flex_child(address_form_field.lens(State::address), 1.0)
        .with_spacer(theme::GUTTER_SIZE)
        .with_child(
            spinner
                .lens(AddressState::request_state_lens)
                .lens(State::address),
        )
        .with_child(send_button)
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

impl<W> Controller<AddressValidationState, W> for AddressController
where
    W: Widget<AddressValidationState>,
{
    fn event(
        &mut self,
        child: &mut W,
        ctx: &mut EventCtx,
        event: &Event,
        data: &mut AddressValidationState,
        env: &Env,
    ) {
        if let Event::Command(command) = event {
            if command.is(FINISH_EDIT) {
                if data.is_valid() {
                    ctx.submit_command(command::CONNECT.to(self.body_id));
                }
            }
        }

        child.event(ctx, event, data, env)
    }

    fn lifecycle(
        &mut self,
        child: &mut W,
        ctx: &mut LifeCycleCtx,
        event: &LifeCycle,
        data: &AddressValidationState,
        env: &Env,
    ) {
        if let LifeCycle::WidgetAdded = event {
            if data.is_valid() {
                ctx.submit_command(command::CONNECT.to(self.body_id));
            }
        }

        child.lifecycle(ctx, event, data, env)
    }
}

fn layout_spinner<T>(child: impl Widget<T> + 'static, padding: f64) -> Box<dyn Widget<T>>
where
    T: Data,
{
    child
        .padding(padding)
        .center()
        .fix_size(24.0, 24.0)
        .padding((0.0, 0.0, theme::GUTTER_SIZE, 0.0))
        .boxed()
}
