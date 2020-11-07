mod item;
pub mod request;

use druid::{
    lens,
    widget::{prelude::*, List, ListIter, Scroll, ViewSwitcher},
    ArcStr, Data, Lens, WidgetExt,
};

use crate::{widget::{Expander, ExpanderData}, grpc, json::JsonText, protobuf::ProtobufMethod};

#[derive(Debug, Clone, Data)]
pub struct State {
    items: im::Vector<ListEntryExpanderState<item::State>>,
    request: ListEntryExpanderState<request::State>,
}

#[derive(Debug, Clone, Data, Lens)]
struct ListEntryExpanderState<T> {
    label: ArcStr,
    expanded: bool,
    data: T,
}

#[derive(Debug, Clone, Data)]
enum ListEntryState {
    Item(item::State),
    Request(request::State),
}

#[derive(Debug, Copy, Clone, Data, PartialEq, Eq)]
enum ListEntryStateKind {
    Item,
    Request,
}

pub fn build() -> Box<dyn Widget<State>> {
    Scroll::new(List::new(build_list_entry)).vertical().boxed()
}

fn build_list_entry() -> impl Widget<ListEntryExpanderState<ListEntryState>> {
    let entry = ViewSwitcher::new(
        |data: &ListEntryState, _| match data {
            ListEntryState::Item(_) => ListEntryStateKind::Item,
            ListEntryState::Request(_) => ListEntryStateKind::Request,
        },
        |&kind, _, _| match kind {
            ListEntryStateKind::Item => item::build()
                .lens(ListEntryState::unwrap_item_lens())
                .boxed(),
            ListEntryStateKind::Request => request::build()
                .lens(ListEntryState::unwrap_request_lens())
                .boxed(),
        },
    )
    .expand_width()
    .lens(ListEntryExpanderState::<ListEntryState>::data);

    Expander::new(|ctx, data, _| {
        // TODO
    }, entry)
}

impl State {
    pub fn empty(method: ProtobufMethod) -> Self {
        State {
            items: im::Vector::new(),
            request: ListEntryExpanderState {
                label: ArcStr::from("Unsent request"),
                expanded: true,
                data: request::State::empty(method),
            },
        }
    }

    pub fn with_text(method: ProtobufMethod, request: impl Into<JsonText>, expanded: bool) -> Self {
        State {
            items: im::Vector::new(),
            request: ListEntryExpanderState {
                label: ArcStr::from("Unsent request"),
                expanded,
                data: request::State::with_text(method, request),
            },
        }
    }

    pub fn update(&mut self, result: grpc::ResponseResult) {
        let name = ArcStr::from(format!("Response {}", self.items.len() + 1));
        self.items.push_back(ListEntryExpanderState {
            label: name,
            expanded: true,
            data: item::State::new(result),
        });
    }

    pub(in crate::app) fn request(&self) -> &request::State {
        &self.request.data
    }
}

impl<T> ExpanderData for ListEntryExpanderState<T>
where
    Self: Data,
{
    fn expanded(&self, _: &Env) -> bool {
        self.expanded
    }

    fn toggle_expanded(&mut self, _: &Env) {
        self.expanded = !self.expanded;
    }

    fn with_label<V>(&self, f: impl FnOnce(&ArcStr) -> V) -> V {
        f(&self.label)
    }

    fn with_label_mut<V>(&mut self, f: impl FnOnce(&mut ArcStr) -> V) -> V {
        f(&mut self.label)
    }
}

impl ListIter<ListEntryExpanderState<ListEntryState>> for State {
    fn for_each(&self, mut cb: impl FnMut(&ListEntryExpanderState<ListEntryState>, usize)) {
        self.items.for_each(|item, index| {
            let entry = ListEntryExpanderState {
                label: item.label.clone(),
                expanded: item.expanded,
                data: ListEntryState::Item(item.data.clone()),
            };
            cb(&entry, index)
        });

        let entry = ListEntryExpanderState {
            label: self.request.label.clone(),
            expanded: self.request.expanded,
            data: ListEntryState::Request(self.request.data.clone()),
        };
        cb(&entry, self.items.len());
    }

    fn for_each_mut(
        &mut self,
        mut cb: impl FnMut(&mut ListEntryExpanderState<ListEntryState>, usize),
    ) {
        self.items.for_each_mut(|item, index| {
            let mut entry = ListEntryExpanderState {
                label: item.label.clone(),
                expanded: item.expanded,
                data: ListEntryState::Item(item.data.clone()),
            };
            cb(&mut entry, index);
            item.expanded = entry.expanded;
            debug_assert!(entry.data.unwrap_item().same(&item.data));
        });

        let mut entry = ListEntryExpanderState {
            label: self.request.label.clone(),
            expanded: self.request.expanded,
            data: ListEntryState::Request(self.request.data.clone()),
        };
        cb(&mut entry, self.items.len());
        self.request.expanded = entry.expanded;
        if !entry.data.unwrap_request().same(&self.request.data) {
            self.request.data = entry.data.unwrap_request().clone();
        }
    }

    fn data_len(&self) -> usize {
        self.items.data_len() + 1
    }
}

impl ListEntryState {
    fn unwrap_item(&self) -> &item::State {
        match self {
            ListEntryState::Item(item) => item,
            _ => panic!("expected item"),
        }
    }

    fn unwrap_item_mut(&mut self) -> &mut item::State {
        match self {
            ListEntryState::Item(item) => item,
            _ => panic!("expected item"),
        }
    }

    fn unwrap_request(&self) -> &request::State {
        match self {
            ListEntryState::Request(request) => request,
            _ => panic!("expected request"),
        }
    }

    fn unwrap_request_mut(&mut self) -> &mut request::State {
        match self {
            ListEntryState::Request(request) => request,
            _ => panic!("expected request"),
        }
    }

    fn unwrap_item_lens() -> impl Lens<ListEntryState, item::State> {
        lens::Field::new(ListEntryState::unwrap_item, ListEntryState::unwrap_item_mut)
    }

    fn unwrap_request_lens() -> impl Lens<ListEntryState, request::State> {
        lens::Field::new(
            ListEntryState::unwrap_request,
            ListEntryState::unwrap_request_mut,
        )
    }
}
