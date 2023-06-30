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
    widget::{Either, Label, LineBreaking, Maybe, Spinner, TextBox, ViewSwitcher},
    ArcStr, Data, Env, Insets, TextAlignment, UnitPoint, Widget, WidgetExt,
};

use crate::theme;

pub use self::{
    editable_list::EditableList,
    empty::{empty, Empty},
    expander::ExpanderData,
    form_field::{
        FinishEditController, FormField, ValidationFn, ValidationState, ERROR_MESSAGE, FINISH_EDIT,
        START_EDIT,
    },
    icon::Icon,
    tabs::{TabId, TabLabelState, TabsData, TabsDataChange},
};

pub fn input<T>(placeholder: impl Into<String>) -> impl Widget<T>
where
    T: Data + TextStorage + EditableText,
{
    let mut text_box = TextBox::new();
    text_box.text_mut().borrow_mut().send_notification_on_return = true;
    text_box.text_mut().borrow_mut().send_notification_on_cancel = true;

    theme::text_box_scope(text_box.with_placeholder(placeholder.into()).expand_width())
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
    let mut text_box = TextBox::multiline();
    if editable {
        text_box.text_mut().borrow_mut().send_notification_on_return = false;
        text_box.text_mut().borrow_mut().send_notification_on_cancel = true;
    }

    theme::text_box_scope(
        text_box
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
                    .with_line_break_mode(LineBreaking::WordWrap)
                    .align_vertical(UnitPoint::CENTER),
            )
            .padding(insets)
        },
        empty,
    )
}

pub fn env_error_label<T>(insets: impl Into<Insets>) -> impl Widget<T>
where
    T: Data,
{
    let insets = insets.into();
    Either::new(
        |_: &T, env: &Env| env.try_get(ERROR_MESSAGE).is_ok(),
        theme::error_label_scope(
            Label::new(|_: &T, env: &Env| {
                env.try_get(ERROR_MESSAGE)
                    .unwrap_or_else(|_| ArcStr::from(""))
            })
            .with_text_alignment(TextAlignment::Start)
            .with_line_break_mode(LineBreaking::WordWrap)
            .align_vertical(UnitPoint::LEFT),
        )
        .padding(insets),
        Empty,
    )
}
