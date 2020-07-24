mod app_delegate;
mod channel;
mod data;
mod menus;
mod theme;
mod widgets;

use druid::widget::Flex;
use druid::{AppLauncher, LocalizedString, Size, Widget, WidgetExt, WindowDesc};

use crate::data::AppState;
use crate::widgets::Sidebar;

pub fn main() {
    let state = AppState::new();

    let main_window = WindowDesc::new(make_ui)
        .title(LocalizedString::new("Grpc Client"))
        .menu(menus::make_menu(&state))
        .window_size(Size::new(900.0, 800.0));

    AppLauncher::with_window(main_window)
        .delegate(app_delegate::Delegate::default())
        .configure_env(|env, _| theme::set_env(env))
        .use_simple_logger()
        .launch(state)
        .expect("launch failed");
}

fn make_ui() -> impl Widget<AppState> {
    Flex::row()
        .with_child(Sidebar.fix_width(100.0))
        .with_flex_child(
            Flex::column().with_flex_child(
                channel::make_widget().lens(AppState::channel).padding(16.0),
                1.0,
            ),
            1.0,
        )
}
