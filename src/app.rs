mod address;
mod command;
mod delegate;
mod menu;
mod request;
mod response;
mod sidebar;
mod theme;

use druid::widget::{Flex, Label, Split};
use druid::{AppLauncher, Data, Lens, PlatformError, Widget, WidgetExt, WindowDesc};

pub fn launch() -> Result<(), PlatformError> {
    let main_window = WindowDesc::new(build)
        .title(TITLE)
        .menu(menu::build())
        .window_size((1280.0, 768.0)) // todo store in config
        .with_min_size((640.0, 384.0))
        .resizable(true)
        .show_titlebar(true);

    AppLauncher::with_window(main_window)
        .configure_env(theme::set)
        .delegate(delegate::build())
        .use_simple_logger()
        .launch(State::default())
}

#[derive(Debug, Default, Clone, Data, Lens)]
struct State {
    sidebar: sidebar::State,
    address: address::State,
    request: request::State,
    response: response::State,
}

const TITLE: &'static str = "gRPC Client";

fn build() -> impl Widget<State> {
    let sidebar = sidebar::build().lens(State::sidebar);
    let main = Flex::column()
        .must_fill_main_axis(true)
        .with_child(address::build().lens(State::address))
        .with_spacer(theme::GUTTER_SIZE)
        .with_child(Label::new("Request").align_left())
        .with_spacer(theme::GUTTER_SIZE)
        .with_flex_child(request::build().lens(State::request), 0.5)
        .with_spacer(theme::GUTTER_SIZE)
        .with_child(Label::new("Response").align_left())
        .with_spacer(theme::GUTTER_SIZE)
        .with_flex_child(response::build().lens(State::response), 0.5)
        .padding(theme::GUTTER_SIZE);

    Split::columns(sidebar, main)
        .split_point(0.2)
        .draggable(true)
        .solid_bar(true)
        .boxed()
}
