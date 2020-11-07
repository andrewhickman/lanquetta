mod item;
pub mod request;

use druid::{
    lens,
    widget::{prelude::*, List, ListIter, Scroll, ViewSwitcher},
    Data, Lens, WidgetExt,
};

use crate::{grpc, json::JsonText, protobuf::ProtobufMethod};

#[derive(Debug, Clone, Data)]
pub struct State {
    items: im::Vector<item::State>,
    request: request::State,
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

fn build_list_entry() -> impl Widget<ListEntryState> {
    ViewSwitcher::new(
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
}

impl State {
    pub fn empty(method: ProtobufMethod) -> Self {
        State {
            items: im::Vector::new(),
            request: request::State::empty(method),
        }
    }

    pub fn with_text(method: ProtobufMethod, request: impl Into<JsonText>) -> Self {
        State {
            items: im::Vector::new(),
            request: request::State::with_text(method, request),
        }
    }

    pub fn update(&mut self, result: grpc::ResponseResult) {
        self.items.push_back(item::State::new(result))
    }

    pub(in crate::app) fn request(&self) -> &request::State {
        &self.request
    }
}

impl ListIter<ListEntryState> for State {
    fn for_each(&self, mut cb: impl FnMut(&ListEntryState, usize)) {
        self.items
            .for_each(|item, index| cb(&ListEntryState::Item(item.clone()), index));
        cb(
            &ListEntryState::Request(self.request.clone()),
            self.items.len(),
        );
    }

    fn for_each_mut(&mut self, mut cb: impl FnMut(&mut ListEntryState, usize)) {
        self.items.for_each_mut(|item, index| {
            let mut entry = ListEntryState::Item(item.clone());
            cb(&mut entry, index);
            debug_assert!(entry.unwrap_item().same(&item));
        });

        let mut request = ListEntryState::Request(self.request.clone());
        cb(&mut request, self.items.len());
        if !request.unwrap_request().same(&self.request) {
            self.request = request.unwrap_request().clone();
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
