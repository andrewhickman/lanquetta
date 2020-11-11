mod item;

use druid::{
    widget::{prelude::*, List, Scroll},
    ArcStr, Data, Lens, WidgetExt,
};
use serde::{Deserialize, Serialize};

use crate::{
    grpc,
    widget::{Expander, ExpanderData},
};

#[derive(Debug, Clone, Data, Lens, Serialize, Deserialize)]
#[serde(from = "im::Vector<ItemExpanderState>")]
#[serde(into = "im::Vector<ItemExpanderState>")]
pub struct State {
    items: im::Vector<ItemExpanderState>,
    #[data(ignore)]
    response_count: usize,
    #[data(ignore)]
    request_count: usize,
}

#[derive(Debug, Clone, Data, Lens, Serialize, Deserialize)]
struct ItemExpanderState {
    kind: ItemKind,
    label: ArcStr,
    expanded: bool,
    data: item::State,
}

#[derive(Debug, Clone, Data, Serialize, Deserialize, PartialEq, Eq)]
enum ItemKind {
    Request,
    Response,
}

pub fn build() -> Box<dyn Widget<State>> {
    Scroll::new(List::new(build_list_entry))
        .vertical()
        .expand_height()
        .lens(State::items)
        .boxed()
}

fn build_list_entry() -> impl Widget<ItemExpanderState> {
    let entry = item::build().expand_width().lens(ItemExpanderState::data);

    Expander::new(|_, _, _| unreachable!(), entry)
}

impl State {
    pub fn new() -> Self {
        State {
            items: im::Vector::new(),
            response_count: 0,
            request_count: 0,
        }
    }

    pub fn add_request(&mut self, request: &grpc::Request) {
        self.request_count += 1;
        let name = ArcStr::from(format!("Request {}", self.request_count));
        self.items.push_back(ItemExpanderState {
            label: name,
            expanded: true,
            data: item::State::from_request(request),
            kind: ItemKind::Request,
        });
    }

    pub fn add_response(&mut self, result: &grpc::ResponseResult) {
        self.response_count += 1;
        let name = ArcStr::from(format!("Response {}", self.response_count));
        self.items.push_back(ItemExpanderState {
            label: name,
            expanded: true,
            data: item::State::from_response(result),
            kind: ItemKind::Response,
        });
    }
}

impl ExpanderData for ItemExpanderState {
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

    fn can_close(&self) -> bool {
        false
    }
}

impl From<im::Vector<ItemExpanderState>> for State {
    fn from(items: im::Vector<ItemExpanderState>) -> Self {
        let request_count = items
            .iter()
            .filter(|item| item.kind == ItemKind::Request)
            .count();
        let response_count = items.len() - request_count;

        State {
            items,
            request_count,
            response_count,
        }
    }
}

impl Into<im::Vector<ItemExpanderState>> for State {
    fn into(self) -> im::Vector<ItemExpanderState> {
        self.items
    }
}
