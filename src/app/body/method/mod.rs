mod controller;
mod request;
mod stream;

pub(in crate::app) use self::stream::State as StreamState;

use druid::{
    widget::{prelude::*, Button, CrossAxisAlignment, Flex, Label, Split},
    Data, Lens, WidgetExt,
};

use self::controller::MethodTabController;
use crate::{
    app::{
        body::{address, RequestState},
        command, metadata,
        sidebar::service::ServiceOptions,
    },
    json::JsonText,
    theme,
};

#[derive(Debug, Clone, Data, Lens)]
pub struct MethodTabState {
    #[lens(ignore)]
    #[data(same_fn = "PartialEq::eq")]
    method: prost_reflect::MethodDescriptor,
    #[lens(name = "address_lens")]
    address: address::AddressState,
    #[lens(name = "request_lens")]
    request: request::State,
    #[lens(name = "stream_lens")]
    stream: stream::State,
    #[lens(ignore)]
    service_options: ServiceOptions,
}

pub fn build_body() -> impl Widget<MethodTabState> {
    let id = WidgetId::next();

    Split::rows(
        Flex::column()
            .with_child(build_address_bar(id))
            .with_spacer(theme::BODY_SPACER)
            .with_child(
                Label::new("Request editor")
                    .with_font(theme::font::HEADER_TWO)
                    .lens(MethodTabState::request_lens),
            )
            .with_spacer(theme::BODY_SPACER)
            .with_flex_child(request::build().lens(MethodTabState::request_lens), 1.0)
            .cross_axis_alignment(CrossAxisAlignment::Start)
            .padding(theme::BODY_PADDING),
        Flex::column()
            .with_child(stream::build_header().lens(MethodTabState::stream_lens))
            .with_spacer(theme::BODY_SPACER)
            .with_flex_child(stream::build().lens(MethodTabState::stream_lens), 1.0)
            .cross_axis_alignment(CrossAxisAlignment::Start)
            .padding(theme::BODY_PADDING),
    )
    .min_size(150.0, 100.0)
    .bar_size(2.0)
    .solid_bar(true)
    .draggable(true)
    .controller(MethodTabController::new())
    .with_id(id)
}

fn build_address_bar(body_id: WidgetId) -> impl Widget<MethodTabState> {
    let address_form_field = address::build(body_id);

    let send_button = theme::button_scope(
        Button::dynamic(
            |data: &MethodTabState, _| match data.address.request_state() {
                RequestState::NotStarted | RequestState::ConnectFailed(_) => "Connect".to_owned(),
                RequestState::ConnectInProgress => "Connecting...".to_owned(),
                RequestState::Connected | RequestState::AuthorizationHookFailed(_) => {
                    "Send".to_owned()
                }
                RequestState::SendInProgress if data.method.is_client_streaming() => {
                    "Send".to_owned()
                }
                RequestState::SendInProgress => "Sending...".to_owned(),
                RequestState::AuthorizationHookInProgress => "Authorizing...".to_owned(),
            },
        )
        .on_click(
            move |ctx: &mut EventCtx, data: &mut MethodTabState, _: &Env| {
                debug_assert!(data.can_send() || data.can_connect());
                match data.address.request_state() {
                    RequestState::NotStarted | RequestState::ConnectFailed(_) => {
                        debug_assert!(data.can_connect());
                        ctx.submit_command(command::CONNECT.to(body_id));
                    }
                    RequestState::ConnectInProgress | RequestState::AuthorizationHookInProgress => {
                        unreachable!()
                    }
                    RequestState::Connected
                    | RequestState::SendInProgress
                    | RequestState::AuthorizationHookFailed(_) => {
                        debug_assert!(data.can_send());
                        ctx.submit_command(command::SEND.to(body_id));
                    }
                }
            },
        ),
    )
    .disabled_if(|data: &MethodTabState, _| !data.can_send() && !data.can_connect());

    let finish_button = theme::button_scope(
        Button::dynamic(
            |data: &MethodTabState, _| match data.address.request_state() {
                RequestState::SendInProgress if data.method.is_client_streaming() => {
                    "Finish".to_owned()
                }
                _ => "Disconnect".to_owned(),
            },
        )
        .on_click(
            move |ctx: &mut EventCtx, data: &mut MethodTabState, _: &Env| {
                debug_assert!(data.can_finish() || data.can_disconnect());
                match data.address.request_state() {
                    RequestState::NotStarted | RequestState::ConnectFailed(_) => unreachable!(),
                    RequestState::SendInProgress if data.method.is_client_streaming() => {
                        ctx.submit_command(command::FINISH.to(body_id));
                    }
                    RequestState::ConnectInProgress
                    | RequestState::AuthorizationHookInProgress
                    | RequestState::Connected
                    | RequestState::SendInProgress
                    | RequestState::AuthorizationHookFailed(_) => {
                        ctx.submit_command(command::DISCONNECT.to(body_id));
                    }
                }
            },
        ),
    )
    .disabled_if(|data: &MethodTabState, _| !data.can_finish() && !data.can_disconnect());

    Flex::row()
        .cross_axis_alignment(CrossAxisAlignment::Start)
        .with_flex_child(address_form_field.lens(MethodTabState::address_lens), 1.0)
        .with_child(send_button.fix_width(100.0))
        .with_spacer(theme::BODY_SPACER)
        .with_child(finish_button.fix_width(100.0))
}

impl MethodTabState {
    pub fn empty(method: prost_reflect::MethodDescriptor, service_options: ServiceOptions) -> Self {
        MethodTabState {
            address: address::AddressState::with_options(&service_options),
            stream: stream::State::new(),
            request: request::State::empty(
                method.input(),
                service_options.default_metadata.clone(),
            ),
            service_options,
            method,
        }
    }

    pub fn new(
        method: prost_reflect::MethodDescriptor,
        address: String,
        request: impl Into<JsonText>,
        request_metadata: metadata::State,
        stream: stream::State,
        service_options: ServiceOptions,
    ) -> Self {
        MethodTabState {
            address: address::AddressState::new(address),
            request: request::State::with_text(method.input(), request, request_metadata),
            method,
            stream,
            service_options,
        }
    }

    pub fn method(&self) -> &prost_reflect::MethodDescriptor {
        &self.method
    }

    pub(in crate::app) fn address(&self) -> &address::AddressState {
        &self.address
    }

    pub(in crate::app) fn request(&self) -> &request::State {
        &self.request
    }

    pub(in crate::app) fn stream(&self) -> &stream::State {
        &self.stream
    }

    pub(crate) fn clear_request_history(&mut self) {
        self.stream.clear();
    }

    pub fn can_send(&self) -> bool {
        self.address.is_valid()
            && self.request.is_valid()
            && match self.address.request_state() {
                RequestState::NotStarted
                | RequestState::ConnectInProgress
                | RequestState::AuthorizationHookInProgress
                | RequestState::ConnectFailed(_) => false,
                RequestState::Connected => true,
                RequestState::SendInProgress | RequestState::AuthorizationHookFailed(_) => {
                    self.method.is_client_streaming()
                }
            }
    }

    pub fn can_connect(&self) -> bool {
        self.address.is_valid()
            && match self.address.request_state() {
                RequestState::NotStarted | RequestState::ConnectFailed(_) => true,
                RequestState::Connected
                | RequestState::ConnectInProgress
                | RequestState::AuthorizationHookInProgress
                | RequestState::SendInProgress
                | RequestState::AuthorizationHookFailed(_) => false,
            }
    }

    pub fn can_finish(&self) -> bool {
        matches!(self.address.request_state(), RequestState::SendInProgress)
            && self.method.is_client_streaming()
    }

    pub fn can_disconnect(&self) -> bool {
        match self.address.request_state() {
            RequestState::ConnectInProgress
            | RequestState::AuthorizationHookInProgress
            | RequestState::Connected
            | RequestState::SendInProgress
            | RequestState::AuthorizationHookFailed(_) => true,
            RequestState::NotStarted | RequestState::ConnectFailed(_) => false,
        }
    }

    pub fn service_options(&self) -> &ServiceOptions {
        &self.service_options
    }

    pub fn set_service_options(&mut self, options: ServiceOptions) {
        self.service_options = options;
    }
}
