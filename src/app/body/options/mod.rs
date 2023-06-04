mod controller;

use druid::{
    widget::{prelude::*, Button, Checkbox, CrossAxisAlignment, Flex, Label},
    Data, Lens, WidgetExt,
};
use prost_reflect::ServiceDescriptor;

use crate::{
    app::{
        body::address::{self, AddressState},
        command, metadata,
        sidebar::service::ServiceOptions,
    },
    theme,
};

use self::controller::OptionsTabController;

use super::RequestState;

#[derive(Debug, Clone, Data, Lens)]
pub struct OptionsTabState {
    #[data(same_fn = "PartialEq::eq")]
    #[lens(ignore)]
    service: ServiceDescriptor,
    default_address: AddressState,
    verify_certs: bool,
    default_metadata: metadata::EditableState,
}

pub fn build_body() -> impl Widget<OptionsTabState> {
    let id = WidgetId::next();

    let tls_checkbox = theme::check_box_scope(Checkbox::new("Enable certificate verification"));

    let default_metadata = metadata::build_editable();

    Flex::column()
        .with_child(
            Label::new("Default address")
                .with_font(theme::font::HEADER_TWO)
                .align_left(),
        )
        .with_spacer(theme::BODY_SPACER)
        .with_child(build_address_bar(id))
        .with_spacer(theme::BODY_SPACER)
        .with_child(tls_checkbox.lens(OptionsTabState::verify_certs))
        .with_spacer(theme::BODY_SPACER)
        .with_child(
            Label::new("Default metadata")
                .with_font(theme::font::HEADER_TWO)
                .align_left(),
        )
        .with_spacer(theme::BODY_SPACER)
        .with_child(default_metadata.lens(OptionsTabState::default_metadata))
        .must_fill_main_axis(true)
        .cross_axis_alignment(CrossAxisAlignment::Start)
        .padding(theme::BODY_PADDING)
        .expand_height()
        .controller(OptionsTabController::new())
        .with_id(id)
}

fn build_address_bar(body_id: WidgetId) -> impl Widget<OptionsTabState> {
    let address_form_field = address::build(body_id);

    let send_button = theme::button_scope(Button::new("Connect").on_click(
        move |ctx: &mut EventCtx, _: &mut OptionsTabState, _: &Env| {
            ctx.submit_command(command::CONNECT.to(body_id));
        },
    ))
    .disabled_if(|data: &OptionsTabState, _| !data.can_connect());

    let finish_button = theme::button_scope(Button::new("Disconnect").on_click(
        move |ctx: &mut EventCtx, _: &mut OptionsTabState, _: &Env| {
            ctx.submit_command(command::DISCONNECT.to(body_id));
        },
    ))
    .disabled_if(|data: &OptionsTabState, _| !data.can_disconnect());

    Flex::row()
        .cross_axis_alignment(CrossAxisAlignment::Start)
        .with_flex_child(
            address_form_field.lens(OptionsTabState::default_address),
            1.0,
        )
        .with_child(send_button.fix_width(100.0))
        .with_spacer(theme::BODY_SPACER)
        .with_child(finish_button.fix_width(100.0))
}

impl OptionsTabState {
    pub fn new(service: ServiceDescriptor, options: ServiceOptions) -> Self {
        OptionsTabState {
            service,
            default_address: match options.default_address {
                Some(uri) => AddressState::new(uri.to_string()),
                None => AddressState::default(),
            },
            verify_certs: options.verify_certs,
            default_metadata: metadata::EditableState::new(options.default_metadata),
        }
    }

    pub fn label(&self) -> String {
        format!("{} options", self.service.name())
    }

    pub fn service(&self) -> &ServiceDescriptor {
        &self.service
    }

    pub fn service_options(&self) -> ServiceOptions {
        ServiceOptions {
            default_address: self.default_address.uri().cloned(),
            verify_certs: self.verify_certs,
            default_metadata: self.default_metadata.to_state(),
        }
    }

    pub fn set_service_options(&mut self, options: ServiceOptions) {
        if let Some(default_address) = options.default_address {
            self.default_address.set_uri(&default_address);
        }
        self.verify_certs = options.verify_certs;
    }

    pub fn can_connect(&self) -> bool {
        self.default_address.is_valid()
            && match self.default_address.request_state() {
                RequestState::NotStarted | RequestState::ConnectFailed(_) => true,
                RequestState::Connected
                | RequestState::ConnectInProgress
                | RequestState::Active => false,
            }
    }

    pub fn can_disconnect(&self) -> bool {
        matches!(
            self.default_address.request_state(),
            RequestState::ConnectInProgress | RequestState::Connected | RequestState::Active
        )
    }
}
