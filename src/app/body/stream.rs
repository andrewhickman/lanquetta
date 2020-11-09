mod item;

use druid::{
    widget::{prelude::*, List, Scroll},
    ArcStr, Data, Lens, WidgetExt,
};

use crate::{
    grpc,
    widget::{Expander, ExpanderData},
};

#[derive(Debug, Clone, Data, Lens)]
pub struct State {
    items: im::Vector<ItemExpanderState>,
    response_count: u32,
    request_count: u32,
}

#[derive(Debug, Clone, Data, Lens)]
struct ItemExpanderState {
    label: ArcStr,
    expanded: bool,
    data: item::State,
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
        });
    }

    pub fn add_response(&mut self, result: &grpc::ResponseResult) {
        self.response_count += 1;
        let name = ArcStr::from(format!("Response {}", self.response_count));
        self.items.push_back(ItemExpanderState {
            label: name,
            expanded: true,
            data: item::State::from_response(result),
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
