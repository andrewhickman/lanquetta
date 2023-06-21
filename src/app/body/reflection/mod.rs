use druid::{widget::prelude::*, Lens};

use crate::{app::sidebar::service::ServiceOptions, widget::Empty};

#[derive(Debug, Clone, Data, Lens)]
pub struct ReflectionTabState {
    options: ServiceOptions,
}

pub fn build_body() -> impl Widget<ReflectionTabState> {
    Empty
}

impl ReflectionTabState {
    pub fn new(options: ServiceOptions) -> ReflectionTabState {
        ReflectionTabState { options }
    }

    pub fn can_send(&self) -> bool {
        todo!()
    }

    pub fn service_options(&self) -> &ServiceOptions {
        &self.options
    }
}
