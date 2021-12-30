mod address;
mod controller;
mod request;
mod stream;

pub(in crate::app) use self::{address::State as AddressState, stream::State as StreamState};

use druid::{
    widget::{Flex, Split},
    Data, Lens, Widget, WidgetExt, WidgetId,
};

use self::controller::MethodTabController;
use crate::{grpc::MethodKind, json::JsonText, theme};

#[derive(Debug, Clone, Data, Lens)]
pub struct MethodTabState {
    #[lens(ignore)]
    #[data(same_fn = "PartialEq::eq")]
    method: prost_reflect::MethodDescriptor,
    #[lens(ignore)]
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
            .with_child(address::build(id).lens(MethodTabState::address_lens()))
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

impl MethodTabState {
    pub fn empty(method: prost_reflect::MethodDescriptor) -> Self {
        MethodTabState {
            address: address::AddressState::default(),
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

    pub(in crate::app) fn address_lens() -> impl Lens<MethodTabState, address::State> {
        struct AddressLens;

        impl Lens<MethodTabState, address::State> for AddressLens {
            fn with<V, F: FnOnce(&address::State) -> V>(&self, data: &MethodTabState, f: F) -> V {
                f(&address::State::new(
                    data.address.clone(),
                    MethodKind::for_method(&data.method),
                    data.request.is_valid(),
                ))
            }

            fn with_mut<V, F: FnOnce(&mut address::State) -> V>(
                &self,
                data: &mut MethodTabState,
                f: F,
            ) -> V {
                let mut address_data = address::State::new(
                    data.address.clone(),
                    MethodKind::for_method(&data.method),
                    data.request.is_valid(),
                );
                let result = f(&mut address_data);

                if !data.address.same(address_data.address_state()) {
                    data.address = address_data.into_address_state();
                }

                result
            }
        }

        AddressLens
    }
}
