use std::{mem, str::FromStr, sync::Arc};

use druid::{
    widget::{
        prelude::*, Controller, CrossAxisAlignment, Either, Flex, Label, LineBreaking, Spinner,
        TextBox, ViewSwitcher,
    },
    Data, Env, EventCtx, Lens, Widget, WidgetExt as _,
};
use http::Uri;
use once_cell::sync::Lazy;

use crate::{
    app::{body::RequestState, command, sidebar::service::ServiceOptions, theme},
    widget::{Empty, FormField, Icon, ValidationFn, ValidationState, FINISH_EDIT},
};

type AddressValidationState = ValidationState<String, Uri, String>;

#[derive(Debug, Clone, Data, Lens)]
pub(in crate::app) struct AddressState {
    #[lens(name = "uri_lens")]
    uri: AddressValidationState,
    #[lens(name = "request_state_lens")]
    request_state: RequestState,
}

struct AddressController {
    parent: WidgetId,
}

pub(in crate::app) fn build(parent: WidgetId) -> impl Widget<AddressState> {
    let address_textbox = FormField::text_box(theme::text_box_scope(
        TextBox::new()
            .with_placeholder("http://localhost:80")
            .expand_width(),
    ))
    .controller(AddressController { parent })
    .lens(AddressState::uri_lens);

    let error_label = theme::error_label_scope(
        Label::dynamic(|data: &AddressState, _| {
            if let Err(err) = data.uri.result() {
                err.clone()
            } else if let RequestState::ConnectFailed(err) = data.request_state() {
                err.clone()
            } else {
                String::default()
            }
        })
        .with_line_break_mode(LineBreaking::WordWrap),
    );
    let error = Either::new(
        |data: &AddressState, _| {
            !data.uri.is_pristine_or_valid()
                || matches!(data.request_state(), RequestState::ConnectFailed(_))
        },
        error_label,
        Empty,
    )
    .expand_width();

    let address_form_field = Flex::column().with_child(address_textbox).with_child(error);

    let spinner = ViewSwitcher::new(
        |request_state: &RequestState, _| mem::discriminant(request_state),
        |_, request_state, _| match request_state {
            RequestState::NotStarted => Empty.boxed(),
            RequestState::ConnectInProgress | RequestState::Active => {
                layout_spinner(Spinner::new(), 2.0)
            }
            RequestState::Connected => {
                layout_spinner(Icon::check().with_color(theme::color::BOLD_ACCENT), 0.0)
            }
            RequestState::ConnectFailed(_) => {
                layout_spinner(Icon::close().with_color(theme::color::ERROR), 0.0)
            }
        },
    );

    Flex::row()
        .cross_axis_alignment(CrossAxisAlignment::Start)
        .with_flex_child(address_form_field, 1.0)
        .with_spacer(theme::BODY_SPACER)
        .with_child(spinner.lens(AddressState::request_state_lens))
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

    pub fn with_options(options: &ServiceOptions) -> AddressState {
        match &options.default_address {
            Some(addr) => AddressState::new(addr.to_string()),
            None => AddressState::default(),
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

    pub fn set_uri(&mut self, uri: &Uri) {
        self.uri.with_text_mut(|t| *t = uri.to_string())
    }

    pub fn request_state(&self) -> &RequestState {
        &self.request_state
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
            if command.is(FINISH_EDIT) && data.is_valid() {
                ctx.submit_command(command::CONNECT.to(self.parent));
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
                ctx.submit_command(command::CONNECT.to(self.parent));
            }
        }

        child.lifecycle(ctx, event, data, env)
    }

    fn update(
        &mut self,
        child: &mut W,
        ctx: &mut UpdateCtx,
        old_data: &AddressValidationState,
        data: &AddressValidationState,
        env: &Env,
    ) {
        if old_data.result() != data.result() {
            ctx.submit_command(command::DISCONNECT.to(self.parent));
        }

        child.update(ctx, old_data, data, env)
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
        .padding((0.0, 0.0, theme::BODY_SPACER, 0.0))
        .boxed()
}

static VALIDATE_URI: Lazy<ValidationFn<String, Uri, String>> = Lazy::new(|| Arc::new(validate_uri));

#[allow(clippy::ptr_arg)]
fn validate_uri(s: &String) -> Result<Uri, String> {
    let uri = Uri::from_str(s).map_err(|err| err.to_string())?;
    if uri.scheme().is_none() {
        return Err("URI must have scheme".to_owned());
    }
    Ok(uri)
}
