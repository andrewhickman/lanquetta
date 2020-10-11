mod method;
mod service;

use std::path::Path;

use anyhow::Result;
use druid::{
    widget::{List, ListIter},
    Data, Lens, Widget, WidgetExt,
};

use crate::{
    protobuf::{ProtobufMethod, ProtobufService},
    theme,
};

#[derive(Debug, Default, Clone, Data, Lens)]
pub(in crate::app) struct State {
    services: im::Vector<service::ServiceState>,
    selected: Option<ProtobufMethod>,
}

pub(in crate::app) fn build() -> Box<dyn Widget<State>> {
    List::new(service::build)
        .background(theme::SIDEBAR_BACKGROUND)
        .env_scope(|env, _| theme::set_contrast(env))
        .boxed()
}

impl State {
    pub fn add_from_path(&mut self, path: &Path) -> Result<()> {
        self.services.extend(
            ProtobufService::load(path)?
                .into_iter()
                .map(service::ServiceState::from),
        );
        Ok(())
    }
}

impl ListIter<service::State> for State {
    fn for_each(&self, mut cb: impl FnMut(&service::State, usize)) {
        for (i, service) in self.services.iter().enumerate() {
            let state = service::State::new(self.selected.to_owned(), service.to_owned());
            cb(&state, i);
        }
    }

    fn for_each_mut(&mut self, mut cb: impl FnMut(&mut service::State, usize)) {
        for (i, service) in self.services.iter_mut().enumerate() {
            let mut state = service::State::new(self.selected.to_owned(), service.to_owned());
            cb(&mut state, i);

            if !self.selected.same(&state.selected) {
                self.selected = state.selected;
            }
            if !service.same(&state.service) {
                *service = state.service;
            }
        }
    }

    fn data_len(&self) -> usize {
        self.services.len()
    }
}
