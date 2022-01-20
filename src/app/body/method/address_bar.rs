use druid::{
    widget::{prelude::*, Button, CrossAxisAlignment, Flex},
    Data, Env, EventCtx, Lens, Widget, WidgetExt as _,
};

use crate::{
    app::{
        body::{
            address::{self, AddressState},
            RequestState,
        },
        command, theme,
    },
    grpc::MethodKind,
};

#[derive(Debug, Clone, Data, Lens)]
pub(in crate::app) struct State {
    address: AddressState,
    body_valid: bool,
    method_kind: MethodKind,
}

pub(in crate::app) fn build(body_id: WidgetId) -> impl Widget<State> {
    let address_form_field = address::build(body_id);

    let send_button = theme::button_scope(
        Button::dynamic(|data: &State, _| match data.address.request_state() {
            RequestState::NotStarted | RequestState::ConnectFailed => "Connect".to_owned(),
            RequestState::ConnectInProgress => "Connecting...".to_owned(),
            RequestState::Connected => "Send".to_owned(),
            RequestState::Active if data.method_kind.client_streaming() => "Send".to_owned(),
            RequestState::Active => "Sending...".to_owned(),
        })
        .on_click(move |ctx: &mut EventCtx, data: &mut State, _: &Env| {
            debug_assert!(data.can_send() || data.can_connect());
            match data.address.request_state() {
                RequestState::NotStarted | RequestState::ConnectFailed => {
                    debug_assert!(data.can_connect());
                    ctx.submit_command(command::CONNECT.to(body_id));
                }
                RequestState::ConnectInProgress => unreachable!(),
                RequestState::Connected | RequestState::Active => {
                    debug_assert!(data.can_send());
                    ctx.submit_command(command::SEND.to(body_id));
                }
            }
        }),
    )
    .disabled_if(|data: &State, _| !data.can_send() && !data.can_connect());

    let finish_button = theme::button_scope(
        Button::dynamic(|data: &State, _| match data.address.request_state() {
            RequestState::Active if data.method_kind.client_streaming() => "Finish".to_owned(),
            _ => "Disconnect".to_owned(),
        })
        .on_click(move |ctx: &mut EventCtx, data: &mut State, _: &Env| {
            debug_assert!(data.can_finish() || data.can_disconnect());
            match data.address.request_state() {
                RequestState::NotStarted | RequestState::ConnectFailed => unreachable!(),
                RequestState::Active if data.method_kind.client_streaming() => {
                    ctx.submit_command(command::FINISH.to(body_id));
                }
                RequestState::ConnectInProgress
                | RequestState::Connected
                | RequestState::Active => {
                    ctx.submit_command(command::DISCONNECT.to(body_id));
                }
            }
        }),
    )
    .disabled_if(|data: &State, _| !data.can_finish() && !data.can_disconnect());

    Flex::row()
        .cross_axis_alignment(CrossAxisAlignment::Start)
        .with_flex_child(address_form_field.lens(State::address), 1.0)
        .with_spacer(theme::BODY_SPACER)
        .with_child(send_button.fix_width(100.0))
        .with_spacer(theme::BODY_SPACER)
        .with_child(finish_button.fix_width(100.0))
}

impl State {
    pub fn new(address: AddressState, method_kind: MethodKind, body_valid: bool) -> Self {
        State {
            address,
            method_kind,
            body_valid,
        }
    }

    pub fn address_state(&self) -> &AddressState {
        &self.address
    }

    pub fn into_address_state(self) -> AddressState {
        self.address
    }

    pub fn can_send(&self) -> bool {
        (self.address.request_state() != RequestState::Active
            || self.method_kind.client_streaming())
            && self.address.request_state() != RequestState::NotStarted
            && self.address.request_state() != RequestState::ConnectInProgress
            && self.address.is_valid()
            && self.body_valid
    }

    pub fn can_connect(&self) -> bool {
        self.address.request_state() != RequestState::Active
            && self.address.request_state() != RequestState::ConnectInProgress
            && self.address.is_valid()
    }

    pub fn can_finish(&self) -> bool {
        self.address.request_state() == RequestState::Active && self.method_kind.client_streaming()
    }

    pub fn can_disconnect(&self) -> bool {
        self.address.request_state() == RequestState::ConnectInProgress
            || self.address.request_state() == RequestState::Connected
            || self.address.request_state() == RequestState::Active
    }
}
