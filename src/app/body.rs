mod address;
mod controller;
mod request;
pub mod stream;

use std::{collections::BTreeMap, ops::Bound, sync::Arc};

use druid::{widget::Flex, widget::Label, Data, Lens, Widget, WidgetExt as _, WidgetId};
use iter_set::Inclusion;

use self::controller::TabController;
use crate::{
    json::JsonText,
    protobuf::ProtobufMethod,
    theme,
    widget::{TabId, TabLabelState, Tabs, TabsData, TabsDataChange},
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

#[derive(Debug, Clone, Data, Lens)]
pub struct TabState {
    #[lens(ignore)]
    method: ProtobufMethod,
    #[lens(ignore)]
    address: address::AddressState,
    #[lens(name = "request_lens")]
    request: request::State,
    #[lens(name = "stream_lens")]
    stream: stream::State,
}

pub(in crate::app) fn build() -> impl Widget<State> {
    Tabs::new(build_body)
}

fn build_body() -> impl Widget<TabState> {
    let id = WidgetId::next();
    Flex::column()
        .must_fill_main_axis(true)
        .with_child(address::build(id).lens(TabState::address_lens()))
        .with_spacer(theme::BODY_SPACER)
        .with_child(
            Label::new("Request editor")
                .with_font(theme::font::HEADER_TWO)
                .align_left(),
        )
        .with_spacer(theme::BODY_SPACER)
        .with_flex_child(request::build().lens(TabState::request_lens), 0.0)
        .with_spacer(theme::BODY_SPACER)
        .with_child(stream::build_header().lens(TabState::stream_lens))
        .with_spacer(theme::BODY_SPACER)
        .with_flex_child(stream::build().lens(TabState::stream_lens), 1.0)
        .padding(theme::BODY_PADDING)
        .controller(TabController::new())
        .with_id(id)
}

impl State {
    pub fn new(tabs: BTreeMap<TabId, TabState>, selected: Option<TabId>) -> Self {
        State {
            tabs: Arc::new(tabs),
            selected,
        }
    }

    pub fn select_or_create_tab(&mut self, method: ProtobufMethod) {
        if self
            .with_selected(|_, tab_data| tab_data.method.same(&method))
            .unwrap_or(false)
        {
            return;
        }

        for (&id, tab) in self.tabs.iter() {
            if tab.method.same(&method) {
                self.selected = Some(id);
                return;
            }
        }

        self.create_tab(method)
    }

    pub fn create_tab(&mut self, method: ProtobufMethod) {
        let id = TabId::next();
        self.selected = Some(id);
        Arc::make_mut(&mut self.tabs).insert(id, TabState::empty(method));
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

    pub fn selected_method(&self) -> Option<ProtobufMethod> {
        self.with_selected(|_, tab_data| tab_data.method.clone())
    }

    pub fn tabs(&self) -> impl Iterator<Item = (TabId, &TabState)> {
        self.tabs.iter().map(|(&id, tab)| (id, tab))
    }
}

impl TabState {
    pub fn empty(method: ProtobufMethod) -> Self {
        TabState {
            address: address::AddressState::default(),
            stream: stream::State::new(),
            request: request::State::empty(method.request()),
            method,
        }
    }

    pub fn new(
        method: ProtobufMethod,
        address: String,
        request: impl Into<JsonText>,
        stream: stream::State,
    ) -> Self {
        TabState {
            address: address::AddressState::new(address),
            request: request::State::with_text(method.request(), request),
            method,
            stream,
        }
    }

    pub fn method(&self) -> &ProtobufMethod {
        &self.method
    }

    pub(in crate::app) fn address(&self) -> &address::AddressState {
        &self.address
    }

    pub(in crate::app) fn request(&self) -> &request::State {
        &self.request
    }

    pub(in crate::app) fn stream(&self) -> &stream::State {
        &self.stream
    }

    pub(in crate::app) fn address_lens() -> impl Lens<TabState, address::State> {
        struct AddressLens;

        impl Lens<TabState, address::State> for AddressLens {
            fn with<V, F: FnOnce(&address::State) -> V>(&self, data: &TabState, f: F) -> V {
                f(&address::State::new(
                    data.address.clone(),
                    data.method.kind(),
                    data.request.is_valid(),
                ))
            }

            fn with_mut<V, F: FnOnce(&mut address::State) -> V>(
                &self,
                data: &mut TabState,
                f: F,
            ) -> V {
                let mut address_data = address::State::new(
                    data.address.clone(),
                    data.method.kind(),
                    data.request.is_valid(),
                );
                let result = f(&mut address_data);

                if !data.address.same(address_data.address_state()) {
                    data.address = address_data.into_address_state();
                }

                result
            }
        }

        AddressLens
    }
}

impl TabsData for State {
    type Item = TabState;

    fn selected(&self) -> Option<TabId> {
        self.selected
    }

    fn with_selected<V>(&self, f: impl FnOnce(TabId, &Self::Item) -> V) -> Option<V> {
        self.selected
            .map(|tab_id| f(tab_id, self.tabs.get(&tab_id).unwrap()))
    }

    fn with_selected_mut<V>(&mut self, f: impl FnOnce(TabId, &mut Self::Item) -> V) -> Option<V> {
        self.selected.map(|tab_id| {
            let tab_data = self.tabs.get(&tab_id).unwrap();
            let mut new_tab_data = tab_data.clone();
            let result = f(tab_id, &mut new_tab_data);

            if !tab_data.same(&new_tab_data) {
                *Arc::make_mut(&mut self.tabs).get_mut(&tab_id).unwrap() = new_tab_data;
            }

            result
        })
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
            let label_data = TabLabelState::new(tab_data.method.name().clone(), selected);

            f(tab_id, &label_data);
        }
    }

    fn for_each_label_mut(&mut self, mut f: impl FnMut(TabId, &mut TabLabelState)) {
        for (&tab_id, tab_data) in self.tabs.iter() {
            let selected = self.selected == Some(tab_id);
            let mut label_data = TabLabelState::new(tab_data.method.name().clone(), selected);

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
                    let label_data = TabLabelState::new(tab_data.method.name().clone(), selected);

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
