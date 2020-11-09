use druid::{
    widget::LineBreaking,
    widget::{Flex, Label, Painter, ViewSwitcher},
    ArcStr, Color, Data, Env, Lens, PaintCtx, RenderContext, Widget, WidgetExt as _,
};

use crate::{
    app::command,
    protobuf::{ProtobufMethod, ProtobufMethodKind},
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
    method: ProtobufMethod,
}

pub(in crate::app) fn build() -> Box<dyn Widget<State>> {
    let kind = ViewSwitcher::new(
        |data: &MethodState, _| data.method.kind(),
        |&kind: &ProtobufMethodKind, _, _| match kind {
            ProtobufMethodKind::Unary => Icon::unary().boxed(),
            ProtobufMethodKind::ClientStreaming => Icon::client_streaming().boxed(),
            ProtobufMethodKind::ServerStreaming => Icon::server_streaming().boxed(),
            ProtobufMethodKind::Streaming => Icon::streaming().boxed(),
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
        .background(Painter::new(|ctx, _, env| {
            let color = method_background_color(ctx, env);
            let bounds = ctx.size().to_rect();
            ctx.fill(bounds, &color);
        }))
        .on_click(|ctx, data: &mut State, _| {
            ctx.submit_command(command::SELECT_OR_CREATE_TAB.with(data.method.method.clone()));
        });

    let add = Icon::add()
        .background(Painter::new(|ctx, _, env| {
            let color = method_background_color(ctx, env);
            let bounds = ctx
                .size()
                .to_rounded_rect(env.get(druid::theme::BUTTON_BORDER_RADIUS));
            ctx.fill(bounds, &color);
        }))
        .on_click(|ctx, data: &mut State, _| {
            ctx.submit_command(command::CREATE_TAB.with(data.method.method.clone()));
        })
        .padding((6.0, 3.0));

    Flex::row()
        .with_flex_child(icon_and_label, 1.0)
        .with_child(add)
        .background(theme::SIDEBAR_BACKGROUND)
        .env_scope(|env, data| {
            if data.selected {
                env.set(
                    theme::SIDEBAR_BACKGROUND,
                    theme::color::active(
                        env.get(theme::SIDEBAR_BACKGROUND),
                        env.get(druid::theme::LABEL_COLOR),
                    ),
                );
            }
        })
        .boxed()
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
                f(data.method.name())
            }

            fn with_mut<V, F: FnOnce(&mut ArcStr) -> V>(&self, data: &mut MethodState, f: F) -> V {
                f(&mut data.method.name().clone())
            }
        }

        NameLens
    }

    pub fn method(&self) -> &ProtobufMethod {
        &self.method
    }
}

impl From<ProtobufMethod> for MethodState {
    fn from(method: ProtobufMethod) -> Self {
        MethodState { method }
    }
}

fn method_background_color(ctx: &mut PaintCtx, env: &Env) -> Color {
    let mut color = env.get(theme::SIDEBAR_BACKGROUND);
    if ctx.is_active() {
        color = theme::color::active(color, env.get(druid::theme::LABEL_COLOR));
    } else if ctx.is_hot() {
        color = theme::color::hot(color, env.get(druid::theme::LABEL_COLOR));
    }
    color
}
