use druid::{widget::Controller, Data, Env, UpdateCtx, Widget};
use prost_reflect::ServiceDescriptor;

use crate::app::{command::SET_SERVICE_OPTIONS, sidebar::service::ServiceOptions};

#[derive(Debug, Clone, Data)]
pub struct OptionsTabState {
    #[data(same_fn = "PartialEq::eq")]
    service: ServiceDescriptor,
    default_address: String,
}

pub struct OptionsTabController;

pub fn build_body() -> impl Widget<OptionsTabState> {
    druid::widget::Label::new("hello")
}

impl OptionsTabState {
    pub fn new(service: ServiceDescriptor, options: ServiceOptions) -> Self {
        OptionsTabState {
            service,
            default_address: options.default_address,
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
            default_address: self.default_address.clone(),
        }
    }
}

impl<W> Controller<OptionsTabState, W> for OptionsTabController
where
    W: Widget<OptionsTabState>,
{
    fn update(
        &mut self,
        child: &mut W,
        ctx: &mut UpdateCtx,
        old_data: &OptionsTabState,
        data: &OptionsTabState,
        env: &Env,
    ) {
        if !old_data.same(data) {
            ctx.submit_command(
                SET_SERVICE_OPTIONS.with((data.service.clone(), data.service_options())),
            );
            child.update(ctx, old_data, data, env);
        }
    }
}
