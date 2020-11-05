pub mod color;
mod scope;

use druid::{widget::Container, Color, Data, Env, FontDescriptor, FontFamily, Key, Widget};

pub(crate) const GUTTER_SIZE: f64 = 16.0;

pub(crate) const EDITOR_FONT: Key<FontDescriptor> = Key::new("app.editor-font");
pub(crate) const TAB_LABEL_FONT: Key<FontDescriptor> = Key::new("app.tab-label-font");

pub(crate) const SIDEBAR_BACKGROUND: Key<Color> = Key::new("app.sidebar-background");
pub(crate) const TAB_BACKGROUND: Key<Color> = Key::new("app.tab-background");

pub(crate) const INVALID: Key<bool> = Key::new("app.invalid");
pub(crate) const DISABLED: Key<bool> = Key::new("app.disabled");

pub(crate) fn set(env: &mut Env) {
    env.set(druid::theme::PRIMARY_LIGHT, color::SUBTLE_ACCENT);
    env.set(druid::theme::PRIMARY_DARK, color::ACCENT);
    env.set(druid::theme::BORDER_DARK, color::BACKGROUND);
    env.set(druid::theme::BORDER_LIGHT, color::BACKGROUND);

    env.set(druid::theme::LABEL_COLOR, color::TEXT);
    env.set(druid::theme::WINDOW_BACKGROUND_COLOR, color::BACKGROUND);
    env.set(druid::theme::BACKGROUND_LIGHT, color::BACKGROUND);
    env.set(druid::theme::BACKGROUND_DARK, color::BACKGROUND);
    env.set(
        druid::theme::SELECTION_COLOR,
        color::active(color::BACKGROUND, color::TEXT),
    );
    env.set(druid::theme::PLACEHOLDER_COLOR, color::DIM_TEXT);
    env.set(druid::theme::CURSOR_COLOR, color::TEXT);
    env.set(druid::theme::BUTTON_DARK, color::BOLD_ACCENT);
    env.set(druid::theme::BUTTON_LIGHT, color::BOLD_ACCENT);

    env.set(druid::theme::SCROLLBAR_COLOR, color::TEXT.with_alpha(0.38));
    env.set(druid::theme::SCROLLBAR_BORDER_COLOR, color::TEXT);

    env.set(
        EDITOR_FONT,
        FontDescriptor::new(FontFamily::MONOSPACE).with_size(14.0),
    );
    env.set(
        TAB_LABEL_FONT,
        FontDescriptor::new(FontFamily::SANS_SERIF).with_size(16.0),
    );
    env.set(SIDEBAR_BACKGROUND, color::SUBTLE_ACCENT);
    env.set(TAB_BACKGROUND, color::ACCENT);

    env.set(DISABLED, false);
    env.set(INVALID, false);
}

pub(crate) fn set_contrast(env: &mut Env) {
    env.set(druid::theme::BACKGROUND_LIGHT, color::SUBTLE_ACCENT);
    env.set(druid::theme::BACKGROUND_DARK, color::SUBTLE_ACCENT);
    env.set(druid::theme::PRIMARY_LIGHT, color::ACCENT);
    env.set(druid::theme::PRIMARY_DARK, color::ACCENT);
    env.set(druid::theme::LABEL_COLOR, color::ACCENT);
    env.set(druid::theme::CURSOR_COLOR, color::ACCENT);
    env.set(
        druid::theme::SCROLLBAR_COLOR,
        color::BACKGROUND.with_alpha(0.38),
    );
    env.set(druid::theme::SCROLLBAR_BORDER_COLOR, color::BACKGROUND);
}

pub(crate) fn button_scope<T>(child: impl Widget<T>) -> impl Widget<T> {
    scope::new(child, |env, state| {
        if env.get(DISABLED) {
            scope::set_disabled(env, druid::theme::BUTTON_LIGHT);
            scope::set_disabled(env, druid::theme::BUTTON_DARK);
        } else {
            scope::set_hot_active(env, state, druid::theme::BUTTON_LIGHT);
            scope::set_hot_active(env, state, druid::theme::BUTTON_DARK);
        }
    })
}

pub(crate) fn text_box_scope<T>(child: impl Widget<T>) -> impl Widget<T> {
    scope::new(child, |env, state| {
        if !env.get(DISABLED) {
            if env.get(INVALID) {
                env.set(druid::theme::BORDER_DARK, color::ERROR);
                env.set(druid::theme::BORDER_LIGHT, color::ERROR);
                env.set(druid::theme::PRIMARY_DARK, color::ERROR);
                env.set(druid::theme::PRIMARY_LIGHT, color::ERROR);
            }

            scope::set_hot(env, state, druid::theme::BORDER_LIGHT);
            scope::set_hot(env, state, druid::theme::BORDER_DARK);
        }
    })
}

pub(crate) fn error_label_scope<T: Data>(child: impl Widget<T> + 'static) -> impl Widget<T> {
    Container::new(child)
        .border(color::ERROR, 1.0)
        .background(color::ERROR.with_alpha(0.38))
        .rounded(druid::theme::TEXTBOX_BORDER_RADIUS)
}
