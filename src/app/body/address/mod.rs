use std::{str::FromStr, sync::Arc};

use druid::{
    widget::{prelude::*, Controller, CrossAxisAlignment, Flex},
    ArcStr, Data, Env, EventCtx, Insets, Lens, Widget, WidgetExt as _,
};
use http::Uri;
use once_cell::sync::Lazy;

use crate::{
    app::{body::RequestState, command, sidebar::service::ServiceOptions, theme},
    lens,
    theme::BODY_PADDING,
    widget::{
        error_label, input, state_icon, FormField, StateIcon, ValidationFn, ValidationState,
        FINISH_EDIT,
    },
};

type AddressValidationState = ValidationState<String, Uri>;

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
    let address_textbox = FormField::text_box(input("http://localhost:80"))
        .controller(AddressController { parent })
        .lens(AddressState::uri_lens);

    let error = error_label(Insets::ZERO)
        .expand_width()
        .lens(lens::Project::new(|data: &AddressState| {
            data.display_error()
        }));

    let address_form_field = Flex::column().with_child(address_textbox).with_child(error);

    let spinner = state_icon((0.0, 0.0, BODY_PADDING, 0.0))
        .lens(lens::Project::new(|data: &AddressState| data.state_icon()));

    Flex::row()
        .cross_axis_alignment(CrossAxisAlignment::Start)
        .with_flex_child(address_form_field, 1.0)
        .with_spacer(theme::BODY_SPACER)
        .with_child(spinner)
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

    pub fn display_error(&self) -> Option<ArcStr> {
        if let Some(err) = self.uri.display_error() {
            Some(err)
        } else if let RequestState::ConnectFailed(err)
        | RequestState::AuthorizationHookFailed(err) = self.request_state()
        {
            Some(err.clone())
        } else {
            None
        }
    }

    pub fn state_icon(&self) -> StateIcon {
        match self.request_state {
            RequestState::NotStarted => StateIcon::NotStarted,
            RequestState::ConnectInProgress
            | RequestState::AuthorizationHookInProgress
            | RequestState::SendInProgress => StateIcon::InProgress,
            RequestState::Connected => StateIcon::Succeeded,
            RequestState::ConnectFailed(_) | RequestState::AuthorizationHookFailed(_) => {
                StateIcon::Failed
            }
        }
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

static VALIDATE_URI: Lazy<ValidationFn<String, Uri>> = Lazy::new(|| Arc::new(validate_uri));

#[allow(clippy::ptr_arg)]
fn validate_uri(s: &String) -> Result<Uri, ArcStr> {
    let uri = Uri::from_str(s).map_err(|err| err.to_string())?;
    if uri.scheme().is_none() {
        return Err("URI must have scheme".into());
    }
    Ok(uri)
}
