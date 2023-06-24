mod method;
pub(in crate::app) mod service;

use std::{iter::FromIterator, path::Path};

use anyhow::Result;
use druid::{
    widget::Scroll,
    widget::{Flex, Label, LineBreaking, List, ListIter, MainAxisAlignment},
    Data, Lens, Widget, WidgetExt as _,
};
use prost_reflect::ServiceDescriptor;

use crate::{app::command, protoc, theme, widget::Icon};

use self::service::ServiceOptions;

use super::body::CompileOptions;

#[derive(Debug, Default, Clone, Data, Lens)]
pub(in crate::app) struct State {
    services: ServiceListState,
    #[data(same_fn = "PartialEq::eq")]
    selected: Option<prost_reflect::MethodDescriptor>,
}

#[derive(Debug, Default, Clone, Data)]
pub(in crate::app) struct ServiceListState {
    services: im::Vector<service::ServiceState>,
    compile_options: CompileOptions,
}

pub(in crate::app) fn build() -> impl Widget<State> {
    let add_button = Flex::row()
        .with_child(Icon::add().padding(8.0))
        .with_child(
            Label::new("Import services")
                .with_font(theme::font::HEADER_ONE)
                .with_line_break_mode(LineBreaking::Clip),
        )
        .must_fill_main_axis(true)
        .main_axis_alignment(MainAxisAlignment::Start)
        .on_click(|ctx, _: &mut State, _| {
            ctx.submit_command(command::add_file());
        })
        .background(theme::hot_or_active_painter(0.0));

    Scroll::new(
        Flex::column()
            .with_child(List::new(service::build))
            .with_child(add_button)
            .main_axis_alignment(MainAxisAlignment::SpaceBetween),
    )
    .vertical()
    .content_must_fill(true)
    .expand_height()
    .background(druid::theme::BACKGROUND_LIGHT)
    .env_scope(|env, _| theme::set_contrast(env))
}

impl State {
    pub fn new(
        services: ServiceListState,
        selected: Option<prost_reflect::MethodDescriptor>,
    ) -> Self {
        State { services, selected }
    }

    pub fn list_state(&self) -> &ServiceListState {
        &self.services
    }

    pub fn selected_method(&self) -> &Option<prost_reflect::MethodDescriptor> {
        &self.selected
    }

    pub fn into_list_state(self) -> ServiceListState {
        self.services
    }
}

impl ServiceListState {
    pub fn new<T>(services: T, compile_options: CompileOptions) -> Self
    where
        T: IntoIterator<Item = service::ServiceState>,
    {
        ServiceListState {
            services: im::Vector::from_iter(services),
            compile_options,
        }
    }

    pub fn add_from_path(&mut self, path: &Path) -> Result<()> {
        let file = protoc::load_file(path, self.compile_options.includes())?;

        self.services
            .extend(file.services().map(service::ServiceState::from));
        Ok(())
    }

    pub fn services(&self) -> &im::Vector<service::ServiceState> {
        &self.services
    }

    pub fn add_service(&mut self, service: ServiceDescriptor, options: ServiceOptions) {
        self.services
            .push_back(service::ServiceState::new(service.clone(), true, options));
    }

    pub fn remove_service(&mut self, index: usize) -> service::ServiceState {
        self.services.remove(index)
    }

    pub fn service_options(&self, service: &ServiceDescriptor) -> Option<&service::ServiceOptions> {
        for service_state in self.services.iter() {
            if service_state.service() == service {
                return Some(service_state.options());
            }
        }

        None
    }

    pub fn set_service_options(
        &mut self,
        service: &ServiceDescriptor,
        options: &service::ServiceOptions,
    ) {
        for service_state in self.services.iter_mut() {
            if service_state.service() == service {
                service_state.set_options(options.clone());
            }
        }
    }

    pub fn compile_options(&self) -> &CompileOptions {
        &self.compile_options
    }

    pub fn set_compile_options(&mut self, options: CompileOptions) {
        self.compile_options = options
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

            debug_assert!(self.selected == state.selected);
            if !service.same(&state.service) {
                *service = state.service;
            }
        }
    }

    fn data_len(&self) -> usize {
        self.services.services.len()
    }
}
