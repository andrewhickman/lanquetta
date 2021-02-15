mod body;
mod command;
mod config;
mod delegate;
mod menu;
mod serde;
mod sidebar;

use druid::{
    widget::Painter,
    widget::{Either, Flex, Label, Split},
    AppLauncher, Data, Lens, PlatformError, RenderContext, UnitPoint, Widget, WidgetExt as _,
    WindowDesc, WindowId,
};

use self::config::{Config, ConfigController};
use self::menu::MenuController;
use crate::{
    theme,
    widget::{Empty, Icon},
};

pub fn launch() -> Result<(), PlatformError> {
    let config = Config::load();

    let main_window_id = WindowId::next();
    let mut main_window = config
        .window
        .apply(WindowDesc::new(build()))
        .title(TITLE)
        .menu(menu::build(&config.data, main_window_id))
        .with_min_size((407.0, 322.0))
        .resizable(true)
        .show_titlebar(true);
    main_window.id = main_window_id;

    AppLauncher::with_window(main_window)
        .configure_env(|env, _| theme::set(env))
        .delegate(delegate::build())
        .launch(config.data)
}

#[derive(Clone, Debug, Default, Data, Lens)]
struct State {
    sidebar: sidebar::ServiceListState,
    body: body::State,
    error: Option<String>,
}

const TITLE: &str = "gRPC Client";

fn build() -> impl Widget<State> {
    let sidebar = sidebar::build().lens(State::sidebar_lens());
    let body = body::build().lens(State::body);

    let error = Either::new(
        |data: &Option<String>, _| data.is_some(),
        theme::error_label_scope(
            Flex::row()
                .with_flex_child(
                    Label::dynamic(|data: &Option<String>, _| {
                        data.as_ref().cloned().unwrap_or_default()
                    })
                    .align_horizontal(UnitPoint::CENTER),
                    1.0,
                )
                .with_child(
                    Icon::close()
                        .background(Painter::new(|ctx, _, env| {
                            let color = theme::color::ERROR.with_alpha(0.38);
                            let bounds = ctx
                                .size()
                                .to_rounded_rect(env.get(druid::theme::BUTTON_BORDER_RADIUS));
                            if ctx.is_active() {
                                let color =
                                    theme::color::active(color, env.get(druid::theme::LABEL_COLOR));
                                ctx.fill(bounds, &color);
                            } else if ctx.is_hot() {
                                let color =
                                    theme::color::hot(color, env.get(druid::theme::LABEL_COLOR));
                                ctx.fill(bounds, &color);
                            }
                        }))
                        .on_click(|_, data: &mut Option<String>, _| {
                            *data = None;
                        }),
                ),
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
        .controller(ConfigController)
        .controller(MenuController)
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
