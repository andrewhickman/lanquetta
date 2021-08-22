use druid::{
    widget::LineBreaking,
    widget::{Flex, Label, ViewSwitcher},
    ArcStr, Data, Lens, Widget, WidgetExt as _,
};

use crate::{
    app::command,
    theme,
    widget::Icon,
};

#[derive(Debug, Clone, Data, Lens)]
pub(in crate::app) struct State {
    pub selected: bool,
    pub method: MethodState,
}

#[derive(Debug, Clone, Data)]
pub(in crate::app) struct MethodState {
    method: protobuf::Method,
}

pub(in crate::app) fn build() -> impl Widget<State> {
    let kind = ViewSwitcher::new(
        |data: &MethodState, _| data.method.kind(),
        |&kind: &protobuf::MethodKind, _, _| match kind {
            protobuf::MethodKind::Unary => Icon::unary().boxed(),
            protobuf::MethodKind::ClientStreaming => Icon::client_streaming().boxed(),
            protobuf::MethodKind::ServerStreaming => Icon::server_streaming().boxed(),
            protobuf::MethodKind::Streaming => Icon::streaming().boxed(),
        },
    )
    .padding((6.0, 3.0));

    let label = Label::raw()
        .with_font(theme::font::HEADER_TWO)
        .with_text_size(16.0)
        .with_line_break_mode(LineBreaking::Clip)
        .padding((8.0, 4.0))
        .expand_width()
        .lens(MethodState::name());

    let icon_and_label = Flex::row()
        .with_child(kind)
        .with_flex_child(label, 1.0)
        .lens(State::method)
        .background(theme::hot_or_active_painter(0.0))
        .on_click(|ctx, data: &mut State, _| {
            ctx.submit_command(command::SELECT_OR_CREATE_TAB.with(data.method.method.clone()));
        });

    let add = Icon::add()
        .background(theme::hot_or_active_painter(
            druid::theme::BUTTON_BORDER_RADIUS,
        ))
        .on_click(|ctx, data: &mut State, _| {
            ctx.submit_command(command::CREATE_TAB.with(data.method.method.clone()));
        })
        .padding((6.0, 3.0));

    Flex::row()
        .with_flex_child(icon_and_label, 1.0)
        .with_child(add)
        .background(druid::theme::BACKGROUND_LIGHT)
        .env_scope(|env, data| {
            if data.selected {
                env.set(
                    druid::theme::BACKGROUND_LIGHT,
                    theme::color::active(
                        env.get(druid::theme::BACKGROUND_LIGHT),
                        env.get(druid::theme::TEXT_COLOR),
                    ),
                );
            }
        })
}

impl State {
    pub fn new(selected: bool, method: MethodState) -> Self {
        State { selected, method }
    }
}

impl MethodState {
    fn name() -> impl Lens<MethodState, ArcStr> {
        struct NameLens;

        impl Lens<MethodState, ArcStr> for NameLens {
            fn with<V, F: FnOnce(&ArcStr) -> V>(&self, data: &MethodState, f: F) -> V {
                f(&data.method.name())
            }

            fn with_mut<V, F: FnOnce(&mut ArcStr) -> V>(&self, data: &mut MethodState, f: F) -> V {
                f(&mut data.method.name().clone())
            }
        }

        NameLens
    }

    pub fn method(&self) -> &protobuf::Method {
        &self.method
    }
}

impl From<protobuf::Method> for MethodState {
    fn from(method: protobuf::Method) -> Self {
        MethodState { method }
    }
}
