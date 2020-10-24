mod body;
mod header;
mod label;

pub use label::State as TabLabelState;

use druid::{
    widget::{CrossAxisAlignment, Flex},
    Data, Selector, Widget,
};

use self::body::TabsBody;
use self::header::TabsHeader;
use self::label::TabLabel;

pub struct Tabs;

#[derive(Copy, Clone, Data, Debug, PartialOrd, Ord, Eq, PartialEq)]
pub struct TabId(u32);

pub enum TabsDataChange<'a, T> {
    Added,
    Changed(&'a T),
    Removed,
}

pub trait TabsData: Data {
    type Item: Data;

    fn selected(&self) -> Option<TabId>;

    fn with_selected<V>(&self, f: impl FnOnce(TabId, &Self::Item) -> V) -> Option<V>;
    fn with_selected_mut<V>(&mut self, f: impl FnOnce(TabId, &mut Self::Item) -> V) -> Option<V>;

    fn for_each(&self, f: impl FnMut(TabId, &Self::Item));
    fn for_each_mut(&mut self, f: impl FnMut(TabId, &mut Self::Item));
    fn for_each_changed(&self, old: &Self, f: impl FnMut(TabId, TabsDataChange<Self::Item>));

    fn for_each_label(&self, f: impl FnMut(TabId, &TabLabelState));
    fn for_each_label_mut(&mut self, f: impl FnMut(TabId, &mut TabLabelState));
    fn for_each_label_changed(
        &self,
        old: &Self,
        f: impl FnMut(TabId, TabsDataChange<TabLabelState>),
    );

    fn remove(&mut self, id: TabId);
}

const CLOSE_TAB: Selector<TabId> = Selector::new("app.tabs.close-tab");

impl Tabs {
    pub fn new<T, F, W>(build_body: F) -> impl Widget<T>
    where
        T: TabsData,
        F: FnMut(TabId) -> W + 'static,
        W: Widget<T::Item> + 'static,
    {
        Flex::column()
            .must_fill_main_axis(true)
            .cross_axis_alignment(CrossAxisAlignment::Start)
            .with_child(TabsHeader::new())
            .with_flex_child(TabsBody::new(build_body), 1.0)
    }
}

impl TabId {
    pub fn next() -> Self {
        use std::sync::atomic::*;

        static COUNTER: AtomicU32 = AtomicU32::new(0);

        TabId(COUNTER.fetch_add(1, Ordering::Relaxed))
    }
}
