mod color;
mod scope;

pub use scope::new as scope;

use druid::Env;

pub(crate) const GUTTER_SIZE: f64 = 16.0;

pub(crate) fn set(env: &mut Env) {
    env.set(druid::theme::PRIMARY_LIGHT, color::TEXT);
    env.set(druid::theme::PRIMARY_DARK, color::TEXT);
    env.set(druid::theme::BORDER_DARK, color::SUBTLE_ACCENT);
    env.set(druid::theme::BORDER_LIGHT, color::SUBTLE_ACCENT);

    env.set(druid::theme::LABEL_COLOR, color::TEXT);
    env.set(druid::theme::WINDOW_BACKGROUND_COLOR, color::ACCENT);
    env.set(druid::theme::BACKGROUND_LIGHT, color::BACKGROUND);
    env.set(druid::theme::BACKGROUND_DARK, color::BACKGROUND);
    env.set(
        druid::theme::SELECTION_COLOR,
        color::active(color::BACKGROUND),
    );
    env.set(druid::theme::PLACEHOLDER_COLOR, color::DIM_TEXT);
    env.set(druid::theme::CURSOR_COLOR, color::TEXT);
    env.set(druid::theme::BUTTON_DARK, color::BOLD_ACCENT);
    env.set(druid::theme::BUTTON_LIGHT, color::BOLD_ACCENT);
}

pub(crate) fn set_contrast(env: &mut Env) {
    env.set(druid::theme::BACKGROUND_LIGHT, color::SUBTLE_ACCENT);
    env.set(druid::theme::BACKGROUND_DARK, color::SUBTLE_ACCENT);
    env.set(druid::theme::PRIMARY_LIGHT, color::ACCENT);
    env.set(druid::theme::PRIMARY_DARK, color::ACCENT);
    env.set(druid::theme::LABEL_COLOR, color::ACCENT);
    env.set(druid::theme::CURSOR_COLOR, color::ACCENT);
}

pub(crate) fn set_error(env: &mut Env) {
    env.set(druid::theme::BORDER_DARK, color::ERROR);
    env.set(druid::theme::BORDER_LIGHT, color::ERROR);
    env.set(druid::theme::PRIMARY_DARK, color::ERROR);
    env.set(druid::theme::PRIMARY_LIGHT, color::ERROR);
}
