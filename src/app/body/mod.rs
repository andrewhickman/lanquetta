mod address;
mod compile;
mod method;
mod options;
mod reflection;

pub(in crate::app) use self::{compile::CompileOptions, method::StreamState};

use std::{collections::BTreeMap, mem, ops::Bound, sync::Arc};

use druid::{lens::Field, widget::ViewSwitcher, ArcStr, Data, Lens, Widget, WidgetExt as _};
use iter_set::Inclusion;
use prost_reflect::{MethodDescriptor, ServiceDescriptor};

use self::{
    compile::CompileTabState, method::MethodTabState, options::OptionsTabState,
    reflection::ReflectionTabState,
};
use crate::{
    app::{command, metadata, sidebar::service::ServiceOptions},
    json::JsonText,
    widget::{tabs, TabId, TabLabelState, TabsData, TabsDataChange},
};

#[derive(Debug, Default, Clone, Data)]
pub(in crate::app) struct State {
    tabs: Arc<BTreeMap<TabId, TabState>>,
    selected: Option<TabId>,
}

#[derive(Debug, Clone, Data)]
pub enum RequestState {
    NotStarted,
    ConnectInProgress,
    Connected,
    ConnectFailed(ArcStr),
    AuthorizationHookFailed(ArcStr),
    SendInProgress,
    AuthorizationHookInProgress,
}

#[derive(Debug, Clone, Data)]
#[allow(clippy::large_enum_variant)]
pub enum TabState {
    Method(MethodTabState),
    Options(OptionsTabState),
    Compile(CompileTabState),
    Reflection(ReflectionTabState),
}

pub(in crate::app) fn build() -> impl Widget<State> {
    tabs::new(|| {
        ViewSwitcher::new(
            |state, _| mem::discriminant(state),
            |_, state, _| match state {
                TabState::Method(_) => method::build_body().lens(TabState::method_lens()).boxed(),
                TabState::Options(_) => {
                    options::build_body().lens(TabState::options_lens()).boxed()
                }
                TabState::Compile(_) => {
                    compile::build_body().lens(TabState::compile_lens()).boxed()
                }
                TabState::Reflection(_) => reflection::build_body()
                    .lens(TabState::reflection_lens())
                    .boxed(),
            },
        )
    })
}

impl State {
    pub fn new(tabs: BTreeMap<TabId, TabState>, selected: Option<TabId>) -> Self {
        State {
            tabs: Arc::new(tabs),
            selected,
        }
    }

    pub fn select_or_create_compiler_tab(&mut self, compile_options: &CompileOptions) {
        for (&id, tab) in self.tabs.iter() {
            if matches!(tab, TabState::Compile(_)) {
                self.selected = Some(id);
                return;
            }
        }

        let id = TabId::next();
        self.selected = Some(id);
        Arc::make_mut(&mut self.tabs).insert(id, TabState::new_compile(compile_options));
    }

    pub fn select_or_create_reflection_tab(&mut self) {
        for (&id, tab) in self.tabs.iter() {
            if matches!(tab, TabState::Reflection(_)) {
                self.selected = Some(id);
                return;
            }
        }

        let id = TabId::next();
        self.selected = Some(id);
        Arc::make_mut(&mut self.tabs).insert(id, TabState::empty_reflection());
    }

    pub fn select_or_create_method_tab(
        &mut self,
        method: &MethodDescriptor,
        options: ServiceOptions,
    ) {
        if self
            .with_selected_method(|_, tab_data| tab_data.method() == method)
            .unwrap_or(false)
        {
            return;
        }

        for (&id, tab) in self.tabs.iter() {
            if matches!(tab, TabState::Method(data) if data.method() == method) {
                self.selected = Some(id);
                return;
            }
        }

        self.create_method_tab(method, options)
    }

    pub fn select_or_create_options_tab(
        &mut self,
        service: &ServiceDescriptor,
        options: &ServiceOptions,
    ) {
        if self
            .with_selected_options(|_, tab_data| tab_data.service() == service)
            .unwrap_or(false)
        {
            return;
        }

        for (&id, tab) in self.tabs.iter() {
            if matches!(tab,
                TabState::Options(data) if data.service() == service)
            {
                self.selected = Some(id);
                return;
            }
        }

        self.create_options_tab(service, options)
    }

    pub fn create_method_tab(&mut self, method: &MethodDescriptor, options: ServiceOptions) {
        let id = TabId::next();
        self.selected = Some(id);
        Arc::make_mut(&mut self.tabs).insert(id, TabState::empty_method(method.clone(), options));
    }

    pub fn create_options_tab(&mut self, service: &ServiceDescriptor, options: &ServiceOptions) {
        let id = TabId::next();
        self.selected = Some(id);
        Arc::make_mut(&mut self.tabs)
            .insert(id, TabState::new_options(service.clone(), options.clone()));
    }

    pub fn first_tab(&self) -> Option<TabId> {
        self.tabs.keys().next().copied()
    }

    pub fn last_tab(&self) -> Option<TabId> {
        self.tabs.keys().next_back().copied()
    }

    pub fn remove_service(&mut self, service: &ServiceDescriptor) {
        Arc::make_mut(&mut self.tabs).retain(|_, v| match v {
            TabState::Method(method) => method.method().parent_service() != service,
            TabState::Options(options) => options.service() != service,
            TabState::Compile(_) => true,
            TabState::Reflection(_) => true,
        });
        self.update_selected_after_remove();
    }

    pub fn select_next_tab(&mut self) {
        if let Some(selected) = self.selected {
            if let Some((&next, _)) = self
                .tabs
                .range((Bound::Excluded(selected), Bound::Unbounded))
                .next()
            {
                self.selected = Some(next);
            }
        }
    }

    pub fn select_prev_tab(&mut self) {
        if let Some(selected) = self.selected {
            if let Some((&prev, _)) = self
                .tabs
                .range((Bound::Unbounded, Bound::Excluded(selected)))
                .next_back()
            {
                self.selected = Some(prev);
            }
        }
    }

    pub fn selected_tab(&self) -> Option<TabId> {
        self.selected
    }

    pub fn close_selected_tab(&mut self) {
        if let Some(selected) = self.selected {
            self.remove(selected);
        }
    }

    pub fn clear_request_history(&mut self) {
        self.with_selected_method_mut(|_, tab| tab.clear_request_history());
    }

    pub fn selected_method(&self) -> Option<prost_reflect::MethodDescriptor> {
        self.with_selected_method(|_, tab_data| tab_data.method().clone())
    }

    pub fn with_selected_method<V>(
        &self,
        f: impl FnOnce(TabId, &MethodTabState) -> V,
    ) -> Option<V> {
        self.try_with_selected(|id, data| match data {
            TabState::Method(method) => Some(f(id, method)),
            _ => None,
        })
    }

    pub fn with_selected_method_mut<V>(
        &mut self,
        f: impl FnOnce(TabId, &mut MethodTabState) -> V,
    ) -> Option<V> {
        self.try_with_selected_mut(|id, data| match data {
            TabState::Method(method) => Some(f(id, method)),
            _ => None,
        })
    }

    pub fn with_selected_options<V>(
        &self,
        f: impl FnOnce(TabId, &OptionsTabState) -> V,
    ) -> Option<V> {
        self.try_with_selected(|id, data| match data {
            TabState::Options(options) => Some(f(id, options)),
            _ => None,
        })
    }

    pub fn tabs(&self) -> impl Iterator<Item = (TabId, &TabState)> {
        self.tabs.iter().map(|(&id, tab)| (id, tab))
    }

    fn try_with_selected<V>(&self, f: impl FnOnce(TabId, &TabState) -> Option<V>) -> Option<V> {
        self.selected
            .and_then(|tab_id| f(tab_id, self.tabs.get(&tab_id).unwrap()))
    }

    fn try_with_selected_mut<V>(
        &mut self,
        f: impl FnOnce(TabId, &mut TabState) -> Option<V>,
    ) -> Option<V> {
        self.selected.and_then(|tab_id| {
            let tab_data = self.tabs.get(&tab_id).unwrap();
            let mut new_tab_data = tab_data.clone();
            let result = f(tab_id, &mut new_tab_data);

            if !tab_data.same(&new_tab_data) {
                *Arc::make_mut(&mut self.tabs).get_mut(&tab_id).unwrap() = new_tab_data;
            }

            result
        })
    }

    fn update_selected_after_remove(&mut self) {
        if let Some(selected) = self.selected {
            self.selected = self
                .tabs
                .range(selected..)
                .next()
                .or_else(|| self.tabs.range(..selected).next_back())
                .map(|(&tab_id, _)| tab_id);
        }
    }

    pub fn set_service_options(&mut self, service: &ServiceDescriptor, options: &ServiceOptions) {
        self.for_each_mut(|_, tab| match tab {
            TabState::Method(tab) => {
                if tab.method().parent_service() == service {
                    tab.set_service_options(options.clone());
                }
            }
            TabState::Options(tab) => {
                if tab.service() == service {
                    tab.set_service_options(options.clone());
                }
            }
            TabState::Compile(_) => (),
            TabState::Reflection(_) => (),
        })
    }

    pub fn can_connect(&self) -> bool {
        self.with_selected(|_, tab| match tab {
            TabState::Method(tab) => tab.can_connect(),
            TabState::Options(tab) => tab.can_connect(),
            TabState::Compile(_) => false,
            TabState::Reflection(_) => false,
        })
        .unwrap_or(false)
    }

    pub fn can_send(&self) -> bool {
        self.with_selected(|_, tab| match tab {
            TabState::Method(tab) => tab.can_send(),
            TabState::Options(_) => false,
            TabState::Compile(_) => false,
            TabState::Reflection(tab) => tab.can_send(),
        })
        .unwrap_or(false)
    }

    pub fn can_finish(&self) -> bool {
        self.with_selected(|_, tab| match tab {
            TabState::Method(tab) => tab.can_finish(),
            TabState::Options(_) => false,
            TabState::Compile(_) => false,
            TabState::Reflection(_) => false,
        })
        .unwrap_or(false)
    }

    pub fn can_disconnect(&self) -> bool {
        self.with_selected(|_, tab| match tab {
            TabState::Method(tab) => tab.can_disconnect(),
            TabState::Options(tab) => tab.can_disconnect(),
            TabState::Compile(_) => false,
            TabState::Reflection(_) => false,
        })
        .unwrap_or(false)
    }
}

impl TabState {
    fn empty_method(method: prost_reflect::MethodDescriptor, options: ServiceOptions) -> TabState {
        TabState::Method(MethodTabState::empty(method, options))
    }

    pub fn new_method(
        method: MethodDescriptor,
        address: String,
        request: JsonText,
        request_metadata: metadata::State,
        stream: StreamState,
        service_options: ServiceOptions,
    ) -> Self {
        TabState::Method(MethodTabState::new(
            method,
            address,
            request,
            request_metadata,
            stream,
            service_options,
        ))
    }

    pub fn new_options(service: ServiceDescriptor, options: ServiceOptions) -> TabState {
        TabState::Options(OptionsTabState::new(service, options))
    }

    pub fn new_compile(options: &CompileOptions) -> TabState {
        TabState::Compile(CompileTabState::new(options))
    }

    pub fn new_reflection(options: ServiceOptions) -> TabState {
        TabState::Reflection(ReflectionTabState::new(options))
    }

    pub fn empty_reflection() -> TabState {
        TabState::Reflection(ReflectionTabState::default())
    }

    pub fn label(&self) -> ArcStr {
        match self {
            TabState::Method(method) => method.method().name().into(),
            TabState::Options(options) => options.label(),
            TabState::Compile(_) => ArcStr::from("Compiler options"),
            TabState::Reflection(_) => ArcStr::from("Server reflection"),
        }
    }

    fn method_lens() -> impl Lens<TabState, MethodTabState> {
        Field::new(
            |data| match data {
                TabState::Method(method) => method,
                _ => panic!("expected method data"),
            },
            |data| match data {
                TabState::Method(method) => method,
                _ => panic!("expected method data"),
            },
        )
    }

    fn options_lens() -> impl Lens<TabState, OptionsTabState> {
        Field::new(
            |data| match data {
                TabState::Options(options) => options,
                _ => panic!("expected options data"),
            },
            |data| match data {
                TabState::Options(options) => options,
                _ => panic!("expected options data"),
            },
        )
    }

    fn compile_lens() -> impl Lens<TabState, CompileTabState> {
        Field::new(
            |data| match data {
                TabState::Compile(compile) => compile,
                _ => panic!("expected compile data"),
            },
            |data| match data {
                TabState::Compile(compile) => compile,
                _ => panic!("expected compile data"),
            },
        )
    }

    fn reflection_lens() -> impl Lens<TabState, ReflectionTabState> {
        Field::new(
            |data| match data {
                TabState::Reflection(reflection) => reflection,
                _ => panic!("expected reflection data"),
            },
            |data| match data {
                TabState::Reflection(reflection) => reflection,
                _ => panic!("expected reflection data"),
            },
        )
    }
}

impl TabsData for State {
    type Item = TabState;

    fn selected(&self) -> Option<TabId> {
        self.selected
    }

    fn with_selected<V>(&self, f: impl FnOnce(TabId, &Self::Item) -> V) -> Option<V> {
        self.try_with_selected(|i, t| Some(f(i, t)))
    }

    fn with_selected_mut<V>(&mut self, f: impl FnOnce(TabId, &mut Self::Item) -> V) -> Option<V> {
        self.try_with_selected_mut(|i, t| Some(f(i, t)))
    }

    fn for_each(&self, mut f: impl FnMut(TabId, &Self::Item)) {
        for (&tab_id, tab_data) in self.tabs.iter() {
            f(tab_id, tab_data)
        }
    }

    fn for_each_mut(&mut self, mut f: impl FnMut(TabId, &mut Self::Item)) {
        let mut new_tabs = self.tabs.clone();
        for (&tab_id, tab_data) in self.tabs.iter() {
            let mut new_tab_data = tab_data.clone();
            f(tab_id, &mut new_tab_data);

            if !tab_data.same(&new_tab_data) {
                Arc::make_mut(&mut new_tabs).insert(tab_id, new_tab_data);
            }
        }
        self.tabs = new_tabs;
    }

    fn for_each_changed(&self, old: &Self, mut f: impl FnMut(TabId, TabsDataChange<Self::Item>)) {
        for inclusion in
            iter_set::classify_by_key(old.tabs.iter(), self.tabs.iter(), |(&tab_id, _)| tab_id)
        {
            match inclusion {
                Inclusion::Left((&tab_id, _)) => {
                    f(tab_id, TabsDataChange::Removed);
                }
                Inclusion::Both(_, (&tab_id, tab_data)) => {
                    f(tab_id, TabsDataChange::Changed(tab_data));
                }
                Inclusion::Right((&tab_id, _)) => {
                    f(tab_id, TabsDataChange::Added);
                }
            }
        }
    }

    fn for_each_label(&self, mut f: impl FnMut(TabId, &TabLabelState)) {
        for (&tab_id, tab_data) in self.tabs.iter() {
            let selected = self.selected == Some(tab_id);
            let label_data = TabLabelState::new(tab_data.label(), selected);

            f(tab_id, &label_data);
        }
    }

    fn for_each_label_mut(&mut self, mut f: impl FnMut(TabId, &mut TabLabelState)) {
        for (&tab_id, tab_data) in self.tabs.iter() {
            let selected = self.selected == Some(tab_id);
            let mut label_data = TabLabelState::new(tab_data.label(), selected);

            f(tab_id, &mut label_data);

            if selected != label_data.selected() {
                self.selected = if label_data.selected() {
                    Some(tab_id)
                } else {
                    None
                }
            }
        }
    }

    fn for_each_label_changed(
        &self,
        old: &Self,
        mut f: impl FnMut(TabId, TabsDataChange<TabLabelState>),
    ) {
        for inclusion in
            iter_set::classify_by_key(old.tabs.iter(), self.tabs.iter(), |(&tab_id, _)| tab_id)
        {
            match inclusion {
                Inclusion::Left((&tab_id, _)) => {
                    f(tab_id, TabsDataChange::Removed);
                }
                Inclusion::Both(_, (&tab_id, tab_data)) => {
                    let selected = self.selected == Some(tab_id);
                    let label_data = TabLabelState::new(tab_data.label(), selected);

                    f(tab_id, TabsDataChange::Changed(&label_data));
                }
                Inclusion::Right((&tab_id, _)) => {
                    f(tab_id, TabsDataChange::Added);
                }
            }
        }
    }

    fn remove(&mut self, id: TabId) {
        Arc::make_mut(&mut self.tabs).remove(&id);
        self.update_selected_after_remove();
    }

    fn route_command_to_hidden(&self, cmd: &druid::Command) -> bool {
        !cmd.is(command::CONNECT)
            && !cmd.is(command::DISCONNECT)
            && !cmd.is(command::SEND)
            && !cmd.is(command::FINISH)
    }
}
