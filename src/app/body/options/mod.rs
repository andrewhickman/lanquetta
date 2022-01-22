mod controller;

use druid::{
    widget::{prelude::*, Flex, Label},
    Data, Lens, WidgetExt,
};
use prost_reflect::ServiceDescriptor;

use crate::{
    app::{
        body::address::{self, AddressState},
        sidebar::service::ServiceOptions,
    },
    theme,
};

use self::controller::OptionsTabController;

#[derive(Debug, Clone, Data, Lens)]
pub struct OptionsTabState {
    #[data(same_fn = "PartialEq::eq")]
    #[lens(ignore)]
    service: ServiceDescriptor,
    default_address: AddressState,
}

pub fn build_body() -> impl Widget<OptionsTabState> {
    let id = WidgetId::next();
    let address = address::build(id);

    Flex::column()
        .with_child(
            Label::new("Default address")
                .with_font(theme::font::HEADER_TWO)
                .align_left(),
        )
        .with_spacer(theme::BODY_SPACER)
        .with_child(address)
        .must_fill_main_axis(true)
        .lens(OptionsTabState::default_address)
        .padding(theme::BODY_PADDING)
        .expand_height()
        .controller(OptionsTabController::new())
        .with_id(id)
}

impl OptionsTabState {
    pub fn new(service: ServiceDescriptor, options: ServiceOptions) -> Self {
        OptionsTabState {
            service,
            default_address: match options.default_address {
                Some(uri) => AddressState::new(uri.to_string()),
                None => AddressState::default(),
            },
        }
    }

    pub fn label(&self) -> String {
        format!("{} options", self.service.name())
    }

    pub fn service(&self) -> &ServiceDescriptor {
        &self.service
    }

    pub fn service_options(&self) -> Option<ServiceOptions> {
        self.default_address
            .uri()
            .map(|default_address| ServiceOptions {
                default_address: Some(default_address.clone()),
            })
    }
}
