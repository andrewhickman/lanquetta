use druid::{
    widget::LineBreaking,
    widget::{Flex, Label, ViewSwitcher},
    ArcStr, Data, Lens, Widget, WidgetExt as _,
};

use crate::{app::command, grpc::MethodKind, lens, theme, widget::Icon};

#[derive(Debug, Clone, Data, Lens)]
pub(in crate::app) struct State {
    pub selected: bool,
    pub method: MethodState,
}

#[derive(Debug, Clone, Data)]
pub(in crate::app) struct MethodState {
    #[data(same_fn = "PartialEq::eq")]
    method: prost_reflect::MethodDescriptor,
}

pub(in crate::app) fn build() -> impl Widget<State> {
    let kind = ViewSwitcher::new(
        |data: &MethodState, _| MethodKind::for_method(&data.method),
        |&kind: &MethodKind, _, _| match kind {
            MethodKind::Unary => Icon::unary().boxed(),
            MethodKind::ClientStreaming => Icon::client_streaming().boxed(),
            MethodKind::ServerStreaming => Icon::server_streaming().boxed(),
            MethodKind::Streaming => Icon::streaming().boxed(),
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
            ctx.submit_command(
                command::SELECT_OR_CREATE_METHOD_TAB.with(data.method.method.clone()),
            );
        });

    let add = Icon::add()
        .button(|ctx, data: &mut State, _| {
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
        lens::Project::new(|data: &MethodState| data.method.name().into())
    }

    pub fn method(&self) -> &prost_reflect::MethodDescriptor {
        &self.method
    }
}

impl From<prost_reflect::MethodDescriptor> for MethodState {
    fn from(method: prost_reflect::MethodDescriptor) -> Self {
        MethodState { method }
    }
}
