mod item;

use std::{iter, time::Duration};

use druid::{
    widget::{
        prelude::*, CrossAxisAlignment, Flex, Label, LineBreaking, List, MainAxisAlignment, Scroll,
    },
    ArcStr, Data, Lens, WidgetExt,
};
use serde::{Deserialize, Serialize};

use crate::{
    grpc, theme,
    widget::{expander, ExpanderData, Icon},
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
    duration: ArcStr,
}

#[derive(Debug, Clone, Data, Serialize, Deserialize, PartialEq, Eq)]
enum ItemKind {
    Request,
    Response,
}

pub fn build() -> impl Widget<State> {
    Scroll::new(List::new(build_list_entry))
        .vertical()
        .expand_height()
        .lens(State::items)
}

pub fn build_header() -> impl Widget<State> {
    Flex::row()
        .must_fill_main_axis(true)
        .cross_axis_alignment(CrossAxisAlignment::Center)
        .with_flex_child(
            Label::new("History")
                .with_font(theme::font::HEADER_TWO)
                .expand_width(),
            1.0,
        )
        .with_child(
            Icon::close()
                .background(theme::hot_or_active_painter(
                    druid::theme::BUTTON_BORDER_RADIUS,
                ))
                .on_click(|_, data: &mut State, _| {
                    data.clear();
                }),
        )
}

fn build_list_entry() -> impl Widget<ItemExpanderState> {
    let entry = item::build().expand_width().lens(ItemExpanderState::data);

    let label = Label::raw()
        .with_font(theme::font::HEADER_TWO)
        .with_line_break_mode(LineBreaking::Clip)
        .lens(ItemExpanderState::label);

    let duration = Label::raw()
        .with_font(theme::font::NORMAL)
        .with_line_break_mode(LineBreaking::Clip)
        .lens(ItemExpanderState::duration);

    let copy_item: Box<dyn FnMut(&mut EventCtx, &mut ItemExpanderState, &Env)> =
        Box::new(move |_, data, _| {
            data.data.set_clipboard();
        });

    let expander_label = Flex::row()
        .must_fill_main_axis(true)
        .main_axis_alignment(MainAxisAlignment::SpaceBetween)
        .cross_axis_alignment(CrossAxisAlignment::Center)
        .with_child(label)
        .with_child(duration);

    expander::new(
        expander_label,
        entry,
        iter::once((Icon::copy().with_size((18.0, 18.0)), copy_item)),
    )
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
        for item in self.items.iter_mut() {
            item.expanded = false;
        }

        self.request_count += 1;
        let name = ArcStr::from(format!("Request {}", self.request_count));
        self.items.push_back(ItemExpanderState {
            label: name,
            expanded: true,
            data: item::State::from_request(request),
            kind: ItemKind::Request,
            duration: ArcStr::from(""),
        });
    }

    pub fn add_response(&mut self, result: &grpc::ResponseResult, duration: Option<Duration>) {
        for item in self.items.iter_mut() {
            item.expanded = false;
        }

        self.response_count += 1;
        let name = ArcStr::from(format!("Response {}", self.response_count));
        self.items.push_back(ItemExpanderState {
            label: name,
            expanded: true,
            data: item::State::from_response(result),
            kind: ItemKind::Response,
            duration: duration.map(format_duration).unwrap_or_default().into(),
        });
    }

    pub fn clear(&mut self) {
        self.items.clear();
        self.request_count = 0;
        self.response_count = 0;
    }
}

impl ExpanderData for ItemExpanderState {
    fn expanded(&self, _: &Env) -> bool {
        self.expanded
    }

    fn toggle_expanded(&mut self, _: &Env) {
        self.expanded = !self.expanded;
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

impl From<State> for im::Vector<ItemExpanderState> {
    fn from(state: State) -> im::Vector<ItemExpanderState> {
        state.items
    }
}

fn format_duration(duration: Duration) -> String {
    fn precision(f: f64) -> usize {
        2 - f.log10().floor().min(2.0) as usize
    }

    if duration.as_secs() != 0 {
        let secs = duration.as_secs_f64();
        format!("{:.*} s", precision(secs), secs)
    } else {
        let millis = duration.as_nanos() as f64 / 1_000_000.0;
        format!("{:.*} ms", precision(millis), millis)
    }
}
