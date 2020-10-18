mod address;
mod request;
mod response;

use druid::{Widget, WidgetExt, Data, Lens};
use druid::widget::{Flex, Label};

use crate::theme;

#[derive(Debug, Default, Clone, Data, Lens)]
pub(in crate::app) struct State {
    address: address::State,
    pub request: request::State,
    pub response: response::State,
}

pub(in crate::app) fn build() -> impl Widget<State> {
    Flex::column()
        .must_fill_main_axis(true)
        .with_child(address::build().lens(State::address))
        .with_spacer(theme::GUTTER_SIZE)
        .with_child(Label::new("Request").align_left())
        .with_spacer(theme::GUTTER_SIZE)
        .with_flex_child(request::build().lens(State::request), 0.5)
        .with_spacer(theme::GUTTER_SIZE)
        .with_child(Label::new("Response").align_left())
        .with_spacer(theme::GUTTER_SIZE)
        .with_flex_child(response::build().lens(State::response), 0.5)
        .padding(theme::GUTTER_SIZE)
        .background(theme::TAB_BACKGROUND)
}