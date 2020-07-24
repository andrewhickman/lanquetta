mod app_delegate;
mod data;
mod menus;
mod theme;

use druid::widget::{Align, Flex, Label, TextBox};
use druid::{AppLauncher, Env, LocalizedString, Size, Widget, WidgetExt, WindowDesc};

use crate::data::AppState;

const VERTICAL_WIDGET_SPACING: f64 = 20.0;
const TEXT_BOX_WIDTH: f64 = 200.0;

pub fn main() {
    let state = AppState::new();

    let main_window = WindowDesc::new(make_ui)
        .title(LocalizedString::new("Grpc Client"))
        .menu(menus::make_menu(&state))
        .window_size(Size::new(900.0, 800.0));

    AppLauncher::with_window(main_window)
        .delegate(app_delegate::Delegate::default())
        .configure_env(|env, _| theme::configure_env(env))
        .use_simple_logger()
        .launch(state)
        .expect("launch failed");
}

fn make_ui() -> impl Widget<AppState> {
    let label = Label::new(|data: &AppState, _env: &Env| format!("Hello {}!", data.name));
    let textbox = TextBox::new()
        .with_placeholder("Who are we greeting?")
        .fix_width(TEXT_BOX_WIDTH)
        .lens(AppState::name);

    let layout = Flex::column()
        .with_child(label)
        .with_spacer(VERTICAL_WIDGET_SPACING)
        .with_child(textbox);

    Align::centered(layout)
}
