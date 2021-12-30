mod method;
mod options;

pub(in crate::app) use self::method::StreamState;

use std::{collections::BTreeMap, mem, ops::Bound, sync::Arc};

use druid::{widget::ViewSwitcher, ArcStr, Data, Lens, Widget, WidgetExt as _};
use iter_set::Inclusion;
use prost_reflect::{MethodDescriptor, ServiceDescriptor};

use self::{method::MethodTabState, options::OptionsTabState};
use crate::{
    json::JsonText,
    widget::{tabs, TabId, TabLabelState, TabsData, TabsDataChange},
};

#[derive(Debug, Default, Clone, Data)]
pub(in crate::app) struct State {
    tabs: Arc<BTreeMap<TabId, TabState>>,
    selected: Option<TabId>,
}

#[derive(Debug, Clone, Copy, Data, Eq, PartialEq)]
pub enum RequestState {
    NotStarted,
    ConnectInProgress,
    Connected,
    ConnectFailed,
    Active,
}

#[derive(Debug, Clone, Data)]
#[allow(clippy::large_enum_variant)]
pub enum TabState {
    Method(MethodTabState),
    Options(OptionsTabState),
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

    pub fn select_or_create_method_tab(&mut self, method: prost_reflect::MethodDescriptor) {
        if self
            .with_selected_method(|_, tab_data| tab_data.method() == &method)
            .unwrap_or(false)
        {
            return;
        }

        for (&id, tab) in self.tabs.iter() {
            match tab {
                TabState::Method(data) if data.method() == &method => {
                    self.selected = Some(id);
                    return;
                }
                _ => (),
            }
        }

        self.create_tab(method)
    }

    pub fn create_tab(&mut self, method: prost_reflect::MethodDescriptor) {
        let id = TabId::next();
        self.selected = Some(id);
        Arc::make_mut(&mut self.tabs).insert(id, TabState::empty_method(method));
    }

    pub fn first_tab(&self) -> Option<TabId> {
        self.tabs.keys().next().copied()
    }

    pub fn last_tab(&self) -> Option<TabId> {
        self.tabs.keys().next_back().copied()
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

    pub fn tabs(&self) -> impl Iterator<Item = (TabId, &TabState)> {
        self.tabs.iter().map(|(&id, tab)| (id, tab))
    }

    pub fn with_selected_address<V>(
        &self,
        f: impl FnOnce(&method::AddressState) -> V,
    ) -> Option<V> {
        self.with_selected_method(|_, tab| MethodTabState::address_lens().with(tab, f))
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
}

impl TabState {
    fn empty_method(method: prost_reflect::MethodDescriptor) -> TabState {
        TabState::Method(MethodTabState::empty(method))
    }

    pub fn new_method(
        method: MethodDescriptor,
        address: String,
        request: JsonText,
        stream: StreamState,
    ) -> Self {
        TabState::Method(MethodTabState::new(method, address, request, stream))
    }

    pub fn new_options(service: ServiceDescriptor) -> TabState {
        TabState::Options(OptionsTabState::new(service))
    }

    pub fn label(&self) -> ArcStr {
        match self {
            TabState::Method(method) => method.method().name().into(),
            TabState::Options(options) => options.label().into(),
        }
    }

    fn method_lens() -> impl Lens<TabState, MethodTabState> {
        struct MethodLens;

        impl Lens<TabState, MethodTabState> for MethodLens {
            fn with<V, F: FnOnce(&MethodTabState) -> V>(&self, data: &TabState, f: F) -> V {
                match data {
                    TabState::Method(method) => f(method),
                    _ => panic!("expected method data"),
                }
            }

            fn with_mut<V, F: FnOnce(&mut MethodTabState) -> V>(
                &self,
                data: &mut TabState,
                f: F,
            ) -> V {
                match data {
                    TabState::Method(method) => f(method),
                    _ => panic!("expected method data"),
                }
            }
        }

        MethodLens
    }

    fn options_lens() -> impl Lens<TabState, OptionsTabState> {
        struct OptionsLens;

        impl Lens<TabState, OptionsTabState> for OptionsLens {
            fn with<V, F: FnOnce(&OptionsTabState) -> V>(&self, data: &TabState, f: F) -> V {
                match data {
                    TabState::Options(options) => f(options),
                    _ => panic!("expected options data"),
                }
            }

            fn with_mut<V, F: FnOnce(&mut OptionsTabState) -> V>(
                &self,
                data: &mut TabState,
                f: F,
            ) -> V {
                match data {
                    TabState::Options(options) => f(options),
                    _ => panic!("expected options data"),
                }
            }
        }

        OptionsLens
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

        self.selected = self
            .tabs
            .range(id..)
            .next()
            .or_else(|| self.tabs.range(..id).next_back())
            .map(|(&tab_id, _)| tab_id);
    }
}
