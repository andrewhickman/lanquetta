pub mod color;
pub mod font;
mod scope;

use druid::{
    widget::{Checkbox, Container, Painter},
    Color, Data, Env, FontDescriptor, Key, KeyOrValue, RenderContext, RoundedRectRadii, Widget,
};

pub(crate) const BODY_PADDING: f64 = 16.0;
pub(crate) const BODY_SPACER: f64 = 12.0;
pub(crate) const GRID_NARROW_SPACER: f64 = 2.0;

pub(crate) const EDITOR_FONT: Key<FontDescriptor> = Key::new("app.editor-font");
pub(crate) const TAB_LABEL_FONT: Key<FontDescriptor> = Key::new("app.tab-label-font");

pub(crate) const SELECTED_TAB_BACKGROUND: Key<Color> = Key::new("app.selected-tab-background");
pub(crate) const HIDDEN_TAB_BACKGROUND: Key<Color> = Key::new("app.hidden-tab-background");
pub(crate) const EXPANDER_BACKGROUND: Key<Color> = Key::new("app.expander-background");
pub(crate) const EXPANDER_PADDING: Key<f64> = Key::new("app.expander-padding");
pub(crate) const EXPANDER_CORNER_RADIUS: Key<f64> = Key::new("app.expander-corner-radius");

pub(crate) const INVALID: Key<bool> = Key::new("app.invalid");

pub(crate) fn set(env: &mut Env) {
    env.set(druid::theme::PRIMARY_LIGHT, color::SUBTLE_ACCENT);
    env.set(druid::theme::PRIMARY_DARK, color::SUBTLE_ACCENT);
    env.set(druid::theme::BORDER_DARK, color::BACKGROUND);
    env.set(druid::theme::BORDER_LIGHT, color::BACKGROUND);

    env.set(druid::theme::TEXT_COLOR, color::TEXT);
    env.set(druid::theme::WINDOW_BACKGROUND_COLOR, color::BACKGROUND);
    env.set(druid::theme::BACKGROUND_LIGHT, color::BACKGROUND);
    env.set(druid::theme::BACKGROUND_DARK, color::BACKGROUND);
    env.set(druid::theme::FOREGROUND_LIGHT, color::SUBTLE_ACCENT);
    env.set(druid::theme::FOREGROUND_DARK, color::SUBTLE_ACCENT);
    env.set(
        druid::theme::DISABLED_FOREGROUND_LIGHT,
        color::disabled(color::SUBTLE_ACCENT),
    );
    env.set(
        druid::theme::DISABLED_FOREGROUND_DARK,
        color::disabled(color::SUBTLE_ACCENT),
    );
    env.set(
        druid::theme::SELECTED_TEXT_BACKGROUND_COLOR,
        color::active(color::BACKGROUND, color::TEXT),
    );
    env.set(druid::theme::PLACEHOLDER_COLOR, color::DIM_TEXT);
    env.set(druid::theme::CURSOR_COLOR, color::TEXT);
    env.set(druid::theme::BUTTON_DARK, color::BOLD_ACCENT);
    env.set(druid::theme::BUTTON_LIGHT, color::BOLD_ACCENT);
    env.set(
        druid::theme::DISABLED_BUTTON_DARK,
        color::disabled(color::BOLD_ACCENT),
    );
    env.set(
        druid::theme::DISABLED_BUTTON_LIGHT,
        color::disabled(color::BOLD_ACCENT),
    );
    env.set(
        druid::theme::DISABLED_TEXT_COLOR,
        color::disabled(color::TEXT),
    );

    env.set(druid::theme::SCROLLBAR_COLOR, color::TEXT.with_alpha(0.38));
    env.set(druid::theme::SCROLLBAR_BORDER_COLOR, color::TEXT);

    env.set(EDITOR_FONT, font::CODE);
    env.set(TAB_LABEL_FONT, font::HEADER_TWO);
    env.set(SELECTED_TAB_BACKGROUND, color::ACCENT);
    env.set(HIDDEN_TAB_BACKGROUND, color::BACKGROUND);
    env.set(EXPANDER_BACKGROUND, color::ACCENT);
    env.set(EXPANDER_PADDING, 3.0);
    env.set(EXPANDER_CORNER_RADIUS, 2.0);

    env.set(INVALID, false);
}

pub(crate) fn set_contrast(env: &mut Env) {
    env.set(druid::theme::BACKGROUND_LIGHT, color::SUBTLE_ACCENT);
    env.set(druid::theme::BACKGROUND_DARK, color::SUBTLE_ACCENT);
    env.set(druid::theme::FOREGROUND_LIGHT, color::ACCENT);
    env.set(druid::theme::FOREGROUND_DARK, color::ACCENT);
    env.set(
        druid::theme::DISABLED_FOREGROUND_LIGHT,
        color::disabled(color::ACCENT),
    );
    env.set(
        druid::theme::DISABLED_FOREGROUND_DARK,
        color::disabled(color::ACCENT),
    );
    env.set(druid::theme::FOREGROUND_DARK, color::ACCENT);
    env.set(druid::theme::PRIMARY_LIGHT, color::ACCENT);
    env.set(druid::theme::PRIMARY_DARK, color::ACCENT);
    env.set(druid::theme::TEXT_COLOR, color::ACCENT);
    env.set(druid::theme::CURSOR_COLOR, color::ACCENT);
    env.set(
        druid::theme::SCROLLBAR_COLOR,
        color::BACKGROUND.with_alpha(0.38),
    );
    env.set(druid::theme::SCROLLBAR_BORDER_COLOR, color::BACKGROUND);
}

pub(crate) fn button_scope<T>(child: impl Widget<T>) -> impl Widget<T> {
    scope::new(child, |env, state| {
        scope::set_hot_active(env, state, druid::theme::BUTTON_LIGHT);
        scope::set_hot_active(env, state, druid::theme::BUTTON_DARK);
    })
}

pub(crate) fn text_box_scope<T>(child: impl Widget<T>) -> impl Widget<T> {
    scope::new(child, |env, state| {
        env.set(druid::theme::BACKGROUND_LIGHT, color::BACKGROUND);

        if !state.is_disabled() {
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

pub(crate) fn check_box_scope(child: Checkbox) -> impl Widget<bool> {
    scope::new(child, |env, state| {
        env.set(druid::theme::BACKGROUND_LIGHT, color::BACKGROUND);

        if !state.is_disabled() {
            scope::set_hot(env, state, druid::theme::BORDER_LIGHT);
            scope::set_hot(env, state, druid::theme::BORDER_DARK);
        }
    })
}

pub(crate) fn error_label_scope<T: Data>(child: impl Widget<T> + 'static) -> impl Widget<T> {
    Container::new(child)
        .border(color::ERROR, 1.0)
        .background(color::ERROR.with_alpha(0.38))
        .rounded(2.0)
}

pub(crate) fn hot_or_active_painter<T>(
    border_radius: impl Into<KeyOrValue<RoundedRectRadii>>,
) -> Painter<T> {
    let border_radius = border_radius.into();
    Painter::new(move |ctx, _: &T, env: &Env| {
        let mut color = env.get(druid::theme::BACKGROUND_LIGHT);
        if ctx.is_active() {
            color = color::active(color, env.get(druid::theme::TEXT_COLOR));
        } else if ctx.is_hot() {
            color = color::hot(color, env.get(druid::theme::TEXT_COLOR));
        }
        let bounds = ctx.size().to_rounded_rect(border_radius.resolve(env));
        ctx.fill(bounds, &color);
    })
}
