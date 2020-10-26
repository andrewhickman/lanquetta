mod body;
mod command;
mod delegate;
mod menu;
mod sidebar;

use druid::widget::Split;
use druid::{AppLauncher, Data, Lens, PlatformError, Widget, WidgetExt as _, WindowDesc};

use crate::theme;

pub fn launch() -> Result<(), PlatformError> {
    let main_window = WindowDesc::new(build)
        .title(TITLE)
        .menu(menu::build())
        .window_size((1280.0, 768.0)) // todo store in config
        .with_min_size((640.0, 384.0))
        .resizable(true)
        .show_titlebar(true);

    let app_launcher = AppLauncher::with_window(main_window);
    let event_sink = app_launcher.get_external_handle();
    app_launcher
        .configure_env(|env, _| theme::set(env))
        .delegate(delegate::build(event_sink))
        .use_simple_logger()
        .launch(State::default())
}

#[derive(Debug, Default, Clone, Data, Lens)]
struct State {
    sidebar: sidebar::ServiceListState,
    body: body::State,
}

const TITLE: &'static str = "gRPC Client";

fn build() -> impl Widget<State> {
    let sidebar = sidebar::build().lens(State::sidebar_lens());
    let body = body::build().lens(State::body);

    Split::columns(sidebar, body)
        .split_point(0.2)
        .min_size(100.0)
        .bar_size(2.0)
        .solid_bar(true)
        .draggable(true)
        .boxed()
}

impl State {
    fn sidebar_lens() -> impl Lens<State, sidebar::State> {
        struct SidebarLens;

        impl Lens<State, sidebar::State> for SidebarLens {
            fn with<V, F: FnOnce(&sidebar::State) -> V>(&self, data: &State, f: F) -> V {
                f(&sidebar::State::new(
                    data.sidebar.clone(),
                    data.body.selected_method(),
                ))
            }

            fn with_mut<V, F: FnOnce(&mut sidebar::State) -> V>(
                &self,
                data: &mut State,
                f: F,
            ) -> V {
                let mut sidebar_data =
                    sidebar::State::new(data.sidebar.clone(), data.body.selected_method());
                let result = f(&mut sidebar_data);

                debug_assert!(sidebar_data
                    .selected_method()
                    .same(&data.body.selected_method()));
                if !sidebar_data.list_state().same(&data.sidebar) {
                    data.sidebar = sidebar_data.into_list_state();
                }

                result
            }
        }

        SidebarLens
    }
}
