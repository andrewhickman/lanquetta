mod method;
pub(in crate::app) mod service;

use std::{iter::FromIterator, path::Path};

use anyhow::Result;
use druid::{
    widget::Scroll,
    widget::{prelude::*, Controller, List, ListIter},
    Data, Lens, Widget, WidgetExt as _, WidgetId,
};

use crate::{
    app::command::REMOVE_SERVICE,
    protobuf::{ProtobufMethod, ProtobufService},
    theme,
};

#[derive(Debug, Default, Clone, Data, Lens)]
pub(in crate::app) struct State {
    services: ServiceListState,
    selected: Option<ProtobufMethod>,
}

#[derive(Debug, Default, Clone, Data)]
pub(in crate::app) struct ServiceListState {
    services: im::Vector<service::ServiceState>,
}

struct SidebarController;

pub(in crate::app) fn build() -> impl Widget<State> {
    let sidebar_id = WidgetId::next();
    Scroll::new(List::new(move || service::build(sidebar_id)))
        .vertical()
        .expand_height()
        .env_scope(|env, _| theme::set_contrast(env))
        .background(theme::SIDEBAR_BACKGROUND)
        .controller(SidebarController)
        .with_id(sidebar_id)
}

impl State {
    pub fn new(services: ServiceListState, selected: Option<ProtobufMethod>) -> Self {
        State { services, selected }
    }

    pub fn list_state(&self) -> &ServiceListState {
        &self.services
    }

    pub fn selected_method(&self) -> &Option<ProtobufMethod> {
        &self.selected
    }

    pub fn into_list_state(self) -> ServiceListState {
        self.services
    }
}

impl ServiceListState {
    pub fn add_from_path(&mut self, path: &Path) -> Result<()> {
        self.services.extend(
            ProtobufService::load_file(path)?
                .into_iter()
                .map(service::ServiceState::from),
        );
        Ok(())
    }

    pub fn services(&self) -> &im::Vector<service::ServiceState> {
        &self.services
    }
}

impl FromIterator<service::ServiceState> for ServiceListState {
    fn from_iter<T>(iter: T) -> Self
    where
        T: IntoIterator<Item = service::ServiceState>,
    {
        ServiceListState {
            services: im::Vector::from_iter(iter),
        }
    }
}

impl ListIter<service::State> for State {
    fn for_each(&self, mut cb: impl FnMut(&service::State, usize)) {
        for (i, service) in self.services.services.iter().enumerate() {
            let state = service::State::new(self.selected.to_owned(), service.to_owned(), i);
            cb(&state, i);
        }
    }

    fn for_each_mut(&mut self, mut cb: impl FnMut(&mut service::State, usize)) {
        for (i, service) in self.services.services.iter_mut().enumerate() {
            let mut state = service::State::new(self.selected.to_owned(), service.to_owned(), i);
            cb(&mut state, i);

            debug_assert!(self.selected.same(&state.selected));
            if !service.same(&state.service) {
                *service = state.service;
            }
        }
    }

    fn data_len(&self) -> usize {
        self.services.services.len()
    }
}

impl<W> Controller<State, W> for SidebarController
where
    W: Widget<State>,
{
    fn event(
        &mut self,
        child: &mut W,
        ctx: &mut EventCtx,
        event: &Event,
        data: &mut State,
        env: &Env,
    ) {
        if let Event::Command(command) = event {
            if let Some(&index) = command.get(REMOVE_SERVICE) {
                data.services.services.remove(index);
                return;
            }
        }

        child.event(ctx, event, data, env)
    }
}
