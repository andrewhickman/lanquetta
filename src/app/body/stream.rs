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
        }
    }

    pub fn update(&mut self, result: grpc::ResponseResult) {
        let name = ArcStr::from(format!("Response {}", self.items.len() + 1));
        self.items.push_front(ItemExpanderState {
            label: name,
            expanded: true,
            data: item::State::new(result),
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
