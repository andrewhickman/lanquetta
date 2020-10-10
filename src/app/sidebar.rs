mod method;
mod service;

use std::path::Path;

use anyhow::Result;
use druid::widget::List;
use druid::{Data, Lens, Widget, WidgetExt};

use crate::protobuf::ProtobufService;

use crate::app::theme;

#[derive(Debug, Default, Clone, Data, Lens)]
pub(in crate::app) struct State {
    services: im::Vector<service::State>,
}

pub(in crate::app) fn build() -> Box<dyn Widget<State>> {
    List::new(service::build)
        .background(druid::theme::BACKGROUND_LIGHT)
        .env_scope(|env, _| theme::set_contrast(env))
        .lens(State::services)
        .boxed()
}

impl State {
    pub fn add_from_path(&mut self, path: &Path) -> Result<()> {
        self.services.extend(
            ProtobufService::load(path)?
                .into_iter()
                .map(service::State::from),
        );
        Ok(())
    }
}
