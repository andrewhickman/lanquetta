pub use druid::theme::{FONT_NAME, SELECTION_COLOR};
use druid::{Color, Env, Key};

pub const SIDEBAR_BACKGROUND: Key<Color> = Key::new("grpc-client.sidebar-background");

pub mod color {
    use druid::Color;

    pub const BACKGROUND: Color = Color::rgb8(0x22, 0x36, 0x43);
    pub const TEXT: Color = Color::rgb8(0xfb, 0xf9, 0xf0);
    pub const SUBTLE_ACCENT: Color = Color::rgb8(0x57, 0x9f, 0xb3);
    pub const ACCENT: Color = Color::rgb8(0x1a, 0x22, 0x43);
    pub const BOLD_ACCENT: Color = Color::rgb8(0xe8, 0x5f, 0x24);
}

pub fn configure_env(env: &mut Env) {
    env.set(SIDEBAR_BACKGROUND, color::SUBTLE_ACCENT);
    // env.set(druid::theme::SELECTION_COLOR, colors::HIGHLIGHT_COLOR);
    env.set(druid::theme::LABEL_COLOR, Color::BLACK);
    env.set(druid::theme::WINDOW_BACKGROUND_COLOR, color::ACCENT);
    env.set(druid::theme::BACKGROUND_LIGHT, color::BACKGROUND);
    env.set(druid::theme::BACKGROUND_DARK, color::SUBTLE_ACCENT);
    env.set(druid::theme::BUTTON_DARK, color::BOLD_ACCENT);
    env.set(druid::theme::BUTTON_LIGHT, color::SUBTLE_ACCENT);
    env.set(druid::theme::LABEL_COLOR, color::TEXT);
}
