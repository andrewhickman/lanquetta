use std::sync::Arc;

use druid::{
    widget::{CrossAxisAlignment, Flex, Label, LineBreaking, List, TextBox},
    Data, Lens, Widget, WidgetExt,
};

use crate::{theme, widget::Icon};

pub type State = Arc<Vec<Entry>>;

#[derive(Debug, Default, Clone, Data, Lens)]
pub struct Entry {
    pub key: String,
    pub value: String,
}

const GRID_NARROW_SPACER: f64 = 2.0;

pub(in crate::app) fn build() -> impl Widget<State> {
    List::new(build_row).with_spacing(GRID_NARROW_SPACER)
}

pub(in crate::app) fn build_editable() -> impl Widget<State> {
    Flex::column()
        .must_fill_main_axis(true)
        .with_child(build())
        .with_spacer(GRID_NARROW_SPACER)
        .with_child(build_add_button())
}

fn build_row() -> impl Widget<Entry> {
    Flex::row()
        .cross_axis_alignment(CrossAxisAlignment::Fill)
        .with_flex_child(
            theme::text_box_scope(TextBox::<String>::new().expand_width()).lens(Entry::key),
            0.33,
        )
        .with_spacer(GRID_NARROW_SPACER)
        .with_flex_child(
            theme::text_box_scope(TextBox::<String>::new().expand_width()).lens(Entry::value),
            0.67,
        )
}

fn build_add_button() -> impl Widget<State> {
    Flex::row()
        .with_child(Icon::add().padding(3.0))
        .with_child(
            Label::new("Add metadata")
                .with_font(theme::font::HEADER_TWO)
                .with_line_break_mode(LineBreaking::Clip),
        )
        .must_fill_main_axis(true)
        .on_click(|_, state: &mut State, _| {
            Arc::make_mut(state).push(Entry::default());
        })
        .background(theme::hot_or_active_painter(
            druid::theme::BUTTON_BORDER_RADIUS,
        ))
}
