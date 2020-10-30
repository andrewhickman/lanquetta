mod body;
mod command;
mod config;
mod delegate;
mod menu;
mod sidebar;

use druid::widget::Split;
use druid::{AppLauncher, Data, Lens, PlatformError, Widget, WidgetExt as _};

use self::config::Config;
use crate::theme;

pub fn launch() -> Result<(), PlatformError> {
    let main_window = Config::load()
        .make_window(build)
        .title(TITLE)
        .menu(menu::build())
        .with_min_size((640.0, 384.0))
        .resizable(true)
        .show_titlebar(true);

    AppLauncher::with_window(main_window)
        .configure_env(|env, _| theme::set(env))
        .delegate(delegate::build())
        .launch(State::default())
}

#[derive(Clone, Debug, Default, Data, Lens)]
struct State {
    sidebar: sidebar::ServiceListState,
    body: body::State,
}

const TITLE: &str = "gRPC Client";

fn build() -> impl Widget<State> {
    let sidebar = sidebar::build().lens(State::sidebar_lens());
    let body = body::build().lens(State::body);

    Split::columns(sidebar, body)
        .split_point(0.2)
        .min_size(100.0)
        .bar_size(2.0)
        .solid_bar(true)
        .draggable(true)
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
