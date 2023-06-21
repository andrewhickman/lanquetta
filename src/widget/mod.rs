pub mod expander;
pub mod tabs;
pub mod update_queue;

mod empty;
mod form_field;
mod icon;

use druid::text::{EditableText, TextStorage};
use druid::widget::{Label, LineBreaking, Maybe, TextBox};
use druid::{ArcStr, Data, Env, Insets, TextAlignment, Widget, WidgetExt};

use crate::{lens, theme};

pub use self::empty::Empty;
pub use self::expander::ExpanderData;
pub use self::form_field::{
    FinishEditController, FormField, ValidationFn, ValidationState, FINISH_EDIT,
};
pub use self::icon::Icon;
pub use self::tabs::{TabId, TabLabelState, TabsData, TabsDataChange};

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

pub fn error_label<T>(
    selector: impl Fn(&T) -> Option<ArcStr>,
    insets: impl Into<Insets>,
) -> impl Widget<T>
where
    T: Data,
{
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
    .lens(lens::Project::new(selector))
}
