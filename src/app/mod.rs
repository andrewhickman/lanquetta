mod body;
mod command;
mod config;
mod delegate;
mod menu;
mod metadata;
mod serde;
mod sidebar;

use druid::{
    widget::Painter,
    widget::{Either, Flex, Split},
    AppLauncher, ArcStr, Data, Insets, Lens, PlatformError, RenderContext, Widget, WidgetExt as _,
    WindowDesc,
};

use self::config::{Config, ConfigController};
use crate::{
    theme,
    widget::{error_label, Empty, Icon},
};

pub fn launch() -> Result<(), PlatformError> {
    let config = Config::load();

    let main_window = config
        .window
        .apply(WindowDesc::new(build()))
        .title(TITLE)
        .menu(menu::build)
        .with_min_size((407.0, 322.0))
        .resizable(true)
        .show_titlebar(true);

    AppLauncher::with_window(main_window)
        .configure_env(|env, _| theme::set(env))
        .delegate(delegate::build())
        .launch(config.data)
}

#[derive(Clone, Debug, Default, Data, Lens)]
struct State {
    sidebar: sidebar::ServiceListState,
    body: body::State,
    error: Option<ArcStr>,
}

const TITLE: &str = "Lanquetta";

fn build() -> impl Widget<State> {
    let sidebar = sidebar::build().lens(State::sidebar_lens());
    let body = body::build().lens(State::body);

    let error = Either::new(
        |data: &Option<ArcStr>, _| data.is_some(),
        Flex::row()
            .with_flex_child(error_label(Insets::ZERO).expand_width(), 1.0)
            .with_child(
                Icon::close()
                    .background(Painter::new(|ctx, _, env| {
                        let color = theme::color::ERROR.with_alpha(0.38);
                        let bounds = ctx
                            .size()
                            .to_rounded_rect(env.get(druid::theme::BUTTON_BORDER_RADIUS));
                        if ctx.is_active() {
                            let color =
                                theme::color::active(color, env.get(druid::theme::TEXT_COLOR));
                            ctx.fill(bounds, &color);
                        } else if ctx.is_hot() {
                            let color = theme::color::hot(color, env.get(druid::theme::TEXT_COLOR));
                            ctx.fill(bounds, &color);
                        }
                    }))
                    .on_click(|_, data: &mut Option<ArcStr>, _| {
                        *data = None;
                    }),
            ),
        Empty,
    );
    let split = Split::columns(sidebar, body)
        .split_point(0.2)
        .min_size(0.0, 200.0)
        .bar_size(2.0)
        .solid_bar(true)
        .draggable(true);
    Flex::column()
        .with_child(error.lens(State::error))
        .with_flex_child(split, 1.0)
        .controller(ConfigController::new())
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

                debug_assert!(sidebar_data.selected_method() == &data.body.selected_method());
                if !sidebar_data.list_state().same(&data.sidebar) {
                    data.sidebar = sidebar_data.into_list_state();
                }

                result
            }
        }

        SidebarLens
    }
}
