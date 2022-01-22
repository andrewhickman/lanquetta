mod controller;
mod request;
mod stream;

pub(in crate::app) use self::stream::State as StreamState;

use druid::{
    widget::{prelude::*, Button, CrossAxisAlignment, Flex, Split},
    Data, Lens, WidgetExt,
};

use self::controller::MethodTabController;
use crate::{
    app::{
        body::{address, RequestState},
        command,
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
}

pub fn build_body() -> impl Widget<MethodTabState> {
    let id = WidgetId::next();

    Split::rows(
        Flex::column()
            .with_child(build_address_bar(id))
            .with_spacer(theme::BODY_SPACER)
            .with_child(request::build_header().lens(MethodTabState::request_lens))
            .with_spacer(theme::BODY_SPACER)
            .with_flex_child(request::build().lens(MethodTabState::request_lens), 1.0)
            .padding(theme::BODY_PADDING),
        Flex::column()
            .with_child(stream::build_header().lens(MethodTabState::stream_lens))
            .with_spacer(theme::BODY_SPACER)
            .with_flex_child(stream::build().lens(MethodTabState::stream_lens), 1.0)
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
                RequestState::Connected => "Send".to_owned(),
                RequestState::Active if data.method.is_client_streaming() => "Send".to_owned(),
                RequestState::Active => "Sending...".to_owned(),
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
                    RequestState::ConnectInProgress => unreachable!(),
                    RequestState::Connected | RequestState::Active => {
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
                RequestState::Active if data.method.is_client_streaming() => "Finish".to_owned(),
                _ => "Disconnect".to_owned(),
            },
        )
        .on_click(
            move |ctx: &mut EventCtx, data: &mut MethodTabState, _: &Env| {
                debug_assert!(data.can_finish() || data.can_disconnect());
                match data.address.request_state() {
                    RequestState::NotStarted | RequestState::ConnectFailed(_) => unreachable!(),
                    RequestState::Active if data.method.is_client_streaming() => {
                        ctx.submit_command(command::FINISH.to(body_id));
                    }
                    RequestState::ConnectInProgress
                    | RequestState::Connected
                    | RequestState::Active => {
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
        .with_spacer(theme::BODY_SPACER)
        .with_child(send_button.fix_width(100.0))
        .with_spacer(theme::BODY_SPACER)
        .with_child(finish_button.fix_width(100.0))
}

impl MethodTabState {
    pub fn empty(method: prost_reflect::MethodDescriptor, options: &ServiceOptions) -> Self {
        MethodTabState {
            address: address::AddressState::with_options(options),
            stream: stream::State::new(),
            request: request::State::empty(method.input()),
            method,
        }
    }

    pub fn new(
        method: prost_reflect::MethodDescriptor,
        address: String,
        request: impl Into<JsonText>,
        stream: stream::State,
    ) -> Self {
        MethodTabState {
            address: address::AddressState::new(address),
            request: request::State::with_text(method.input(), request),
            method,
            stream,
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
                | RequestState::ConnectFailed(_) => false,
                RequestState::Connected => true,
                RequestState::Active => self.method.is_client_streaming(),
            }
    }

    pub fn can_connect(&self) -> bool {
        self.address.is_valid()
            && match self.address.request_state() {
                RequestState::NotStarted | RequestState::ConnectFailed(_) => true,
                RequestState::Connected
                | RequestState::ConnectInProgress
                | RequestState::Active => false,
            }
    }

    pub fn can_finish(&self) -> bool {
        matches!(self.address.request_state(), RequestState::Active)
            && self.method.is_client_streaming()
    }

    pub fn can_disconnect(&self) -> bool {
        matches!(
            self.address.request_state(),
            RequestState::ConnectInProgress | RequestState::Connected | RequestState::Active
        )
    }
}
