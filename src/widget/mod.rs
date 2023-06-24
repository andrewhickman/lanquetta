pub mod expander;
pub mod tabs;
pub mod update_queue;

mod editable_list;
mod empty;
mod form_field;
mod icon;

use std::mem;

use druid::{
    text::{EditableText, TextStorage},
    widget::{Label, LineBreaking, Maybe, Spinner, TextBox, ViewSwitcher},
    ArcStr, Data, Env, Insets, TextAlignment, Widget, WidgetExt,
};

use crate::theme;

pub use self::{
    editable_list::EditableList,
    empty::Empty,
    expander::ExpanderData,
    form_field::{FinishEditController, FormField, ValidationFn, ValidationState, FINISH_EDIT},
    icon::Icon,
    tabs::{TabId, TabLabelState, TabsData, TabsDataChange},
};

pub fn input<T>(placeholder: impl Into<String>) -> impl Widget<T>
where
    T: Data + TextStorage + EditableText,
{
    theme::text_box_scope(
        TextBox::new()
            .with_placeholder(placeholder.into())
            .expand_width(),
    )
}

pub fn readonly_input<T>() -> impl Widget<T>
where
    T: Data + TextStorage + EditableText,
{
    theme::text_box_scope(TextBox::new().readonly().expand_width())
}

pub fn code_area<T>(editable: bool) -> impl Widget<T>
where
    T: Data + TextStorage + EditableText,
{
    theme::text_box_scope(
        TextBox::multiline()
            .with_font(theme::EDITOR_FONT)
            .with_editable(editable)
            .expand_width(),
    )
}

#[derive(Data, Copy, Clone, Debug, PartialEq, Eq)]
pub enum StateIcon {
    NotStarted,
    InProgress,
    Succeeded,
    Failed,
}

pub fn state_icon(insets: impl Into<Insets>) -> impl Widget<StateIcon> {
    let insets = insets.into();
    ViewSwitcher::new(
        |state: &StateIcon, _| mem::discriminant(state),
        move |_, request_state, _| match request_state {
            StateIcon::NotStarted => Empty.boxed(),
            StateIcon::InProgress => Spinner::new()
                .padding(2.0)
                .center()
                .fix_size(24.0, 24.0)
                .padding(insets)
                .boxed(),
            StateIcon::Succeeded => Icon::check()
                .with_color(theme::color::BOLD_ACCENT)
                .center()
                .fix_size(24.0, 24.0)
                .padding(insets)
                .boxed(),
            StateIcon::Failed => Icon::close()
                .with_color(theme::color::ERROR)
                .center()
                .fix_size(24.0, 24.0)
                .padding(insets)
                .boxed(),
        },
    )
}

pub fn error_label(insets: impl Into<Insets>) -> impl Widget<Option<ArcStr>> {
    let insets = insets.into();
    Maybe::new(
        move || {
            theme::error_label_scope(
                Label::new(|data: &ArcStr, _: &Env| data.clone())
                    .with_text_alignment(TextAlignment::Start)
                    .with_line_break_mode(LineBreaking::WordWrap),
            )
            .padding(insets)
        },
        || Empty,
    )
}
