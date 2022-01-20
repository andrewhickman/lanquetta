use druid::{
    widget::{prelude::*, Controller, Flex, Label},
    Data, Lens, WidgetExt,
};
use prost_reflect::ServiceDescriptor;

use crate::{
    app::{
        body::address::{self, AddressState},
        command::{self, SET_SERVICE_OPTIONS},
        sidebar::service::ServiceOptions,
    },
    theme,
};

#[derive(Debug, Clone, Data, Lens)]
pub struct OptionsTabState {
    #[data(same_fn = "PartialEq::eq")]
    #[lens(ignore)]
    service: ServiceDescriptor,
    default_address: AddressState,
}

pub struct OptionsTabController;

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
        .controller(OptionsTabController)
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

impl<W> Controller<OptionsTabState, W> for OptionsTabController
where
    W: Widget<OptionsTabState>,
{
    fn event(
        &mut self,
        child: &mut W,
        ctx: &mut EventCtx,
        event: &Event,
        data: &mut OptionsTabState,
        env: &Env,
    ) {
        if let Event::Command(command) = event {
            if command.is(command::CONNECT) {
                tracing::info!("connect!");
                // TODO connect
            } else if command.is(command::DISCONNECT) {
                tracing::info!("disconnect!");
                // TODO disconnect
            }
        }

        child.event(ctx, event, data, env)
    }

    fn update(
        &mut self,
        child: &mut W,
        ctx: &mut UpdateCtx,
        old_data: &OptionsTabState,
        data: &OptionsTabState,
        env: &Env,
    ) {
        if !old_data.same(data) {
            if let Some(service_options) = data.service_options() {
                ctx.submit_command(
                    SET_SERVICE_OPTIONS.with((data.service.clone(), service_options)),
                );
            }
            child.update(ctx, old_data, data, env);
        }
    }
}
