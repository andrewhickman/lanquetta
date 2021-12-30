use druid::{Data, Widget};
use prost_reflect::ServiceDescriptor;

#[derive(Debug, Clone, Data)]
pub struct OptionsTabState {
    #[data(same_fn = "PartialEq::eq")]
    service: ServiceDescriptor,
}

pub fn build_body() -> impl Widget<OptionsTabState> {
    druid::widget::Label::new("hello")
}

impl OptionsTabState {
    pub fn new(service: ServiceDescriptor) -> Self {
        OptionsTabState { service }
    }

    pub fn label(&self) -> String {
        format!("{} options", self.service.name())
    }

    pub fn service(&self) -> ServiceDescriptor {
        self.service.clone()
    }
}
