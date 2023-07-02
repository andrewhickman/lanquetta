mod controller;

use std::sync::Arc;

use druid::{
    widget::{prelude::*, Button, Checkbox, CrossAxisAlignment, Flex, Label, List, Maybe, Scroll},
    Lens, Selector, WidgetExt,
};

use crate::{
    app::{
        body::{
            address::{self, AddressState},
            options::proxy,
        },
        metadata,
        sidebar::service::ServiceOptions,
    },
    theme::{self, BODY_SPACER, GRID_NARROW_SPACER},
    widget::{empty, readonly_input, Icon},
};

use self::controller::ReflectionController;

/// Connect
pub const LIST_SERVICES: Selector = Selector::new("app.body.reflection.list-services");
pub const IMPORT_SERVICE: Selector<String> = Selector::new("app.body.reflection.import-service");

#[derive(Default, Debug, Clone, Data, Lens)]
pub struct ReflectionTabState {
    address: AddressState,
    verify_certs: bool,
    metadata: metadata::EditableState,
    services: Option<Arc<Vec<String>>>,
    proxy: proxy::State,
}

pub fn build_body() -> impl Widget<ReflectionTabState> {
    let id = WidgetId::next();

    let tls_checkbox = theme::check_box_scope(Checkbox::new("Enable certificate verification"));

    Scroll::new(
        Flex::column()
            .with_child(Label::new("Default address").with_font(theme::font::HEADER_TWO))
            .with_spacer(theme::BODY_SPACER)
            .with_child(build_address_bar(id))
            .with_spacer(theme::BODY_SPACER)
            .with_child(tls_checkbox.lens(ReflectionTabState::verify_certs))
            .with_spacer(theme::BODY_SPACER)
            .with_child(Label::new("Metadata").with_font(theme::font::HEADER_TWO))
            .with_spacer(theme::BODY_SPACER)
            .with_child(proxy::build().lens(ReflectionTabState::proxy))
            .with_spacer(theme::BODY_SPACER)
            .with_child(metadata::build_editable().lens(ReflectionTabState::metadata))
            .with_child(
                Maybe::new(move || build_service_list(id), empty)
                    .lens(ReflectionTabState::services),
            )
            .cross_axis_alignment(CrossAxisAlignment::Start)
            .padding(theme::BODY_PADDING)
            .controller(ReflectionController::new())
            .with_id(id),
    )
    .vertical()
    .expand_height()
}

fn build_address_bar(parent: WidgetId) -> impl Widget<ReflectionTabState> {
    let address_form_field = address::build(parent);

    let list_button = theme::button_scope(Button::new("List services").on_click(
        move |ctx: &mut EventCtx, _: &mut ReflectionTabState, _: &Env| {
            ctx.submit_command(LIST_SERVICES.to(parent));
        },
    ))
    .disabled_if(|data: &ReflectionTabState, _| !data.can_send());

    Flex::row()
        .with_flex_child(address_form_field.lens(ReflectionTabState::address), 1.0)
        .with_child(list_button)
        .cross_axis_alignment(CrossAxisAlignment::Start)
}

fn build_service_list(parent: WidgetId) -> impl Widget<Arc<Vec<String>>> {
    Flex::column()
        .with_spacer(BODY_SPACER)
        .with_child(Label::new("Available services").with_font(theme::font::HEADER_TWO))
        .with_spacer(BODY_SPACER)
        .with_child(List::new(move || build_service_row(parent)).with_spacing(GRID_NARROW_SPACER))
        .cross_axis_alignment(CrossAxisAlignment::Start)
}

fn build_service_row(parent: WidgetId) -> impl Widget<String> {
    Flex::row()
        .with_flex_child(readonly_input(), 1.0)
        .with_spacer(GRID_NARROW_SPACER)
        .with_child(
            Icon::add().button(move |ctx: &mut EventCtx, data: &mut String, _| {
                ctx.submit_command(IMPORT_SERVICE.with(data.clone()).to(parent));
            }),
        )
}

impl ReflectionTabState {
    pub fn new(options: ServiceOptions) -> ReflectionTabState {
        ReflectionTabState {
            address: match &options.default_address {
                Some(uri) => AddressState::new(uri.to_string()),
                None => AddressState::default(),
            },
            verify_certs: options.verify_certs,
            metadata: metadata::EditableState::new(options.default_metadata),
            services: None,
            proxy: proxy::State::new(options.proxy),
        }
    }

    pub fn service_options(&self) -> ServiceOptions {
        ServiceOptions {
            default_address: self.address.uri().cloned(),
            verify_certs: self.verify_certs,
            default_metadata: self.metadata.to_state(),
            auth_hook: None,
            proxy: self.proxy.get(),
        }
    }

    pub fn can_send(&self) -> bool {
        self.address.is_valid() && self.metadata.is_valid()
    }
}
