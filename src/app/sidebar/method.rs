use druid::{
    widget::{Flex, Label, Painter},
    ArcStr, Data, FontDescriptor, FontFamily, Lens, RenderContext, Widget, WidgetExt as _,
};

use crate::{app::command, protobuf::ProtobufMethod, theme, widget::Icon};

#[derive(Debug, Clone, Data, Lens)]
pub(in crate::app) struct State {
    pub selected: bool,
    pub method: MethodState,
}

#[derive(Debug, Clone, Data)]
pub(in crate::app) struct MethodState {
    method: ProtobufMethod,
}

struct BackgroundController;

pub(in crate::app) fn build() -> Box<dyn Widget<State>> {
    let label = Label::raw()
        .with_font(FontDescriptor::new(FontFamily::SANS_SERIF))
        .with_text_size(16.0)
        .padding((theme::GUTTER_SIZE, theme::GUTTER_SIZE / 4.0))
        .expand_width()
        .lens(MethodState::name())
        .lens(State::method)
        .background(Painter::new(|ctx, data: &State, env| {
            let mut color = env.get(theme::SIDEBAR_BACKGROUND);
            if ctx.is_active() {
                color = theme::color::active(color, env.get(druid::theme::LABEL_COLOR));
            } else if ctx.is_hot() {
                color = theme::color::hot(color, env.get(druid::theme::LABEL_COLOR));
            }
            let bounds = ctx.size().to_rect();
            ctx.fill(bounds, &color);
        }))
        .on_click(|ctx, data, _| {
            data.selected = true;
            ctx.submit_command(command::SELECT_OR_CREATE_TAB.with(data.method.method.clone()));
        });

    let add = Icon::add()
        .background(Painter::new(|ctx, data: &State, env| {
            let mut color = env.get(theme::SIDEBAR_BACKGROUND);
            if ctx.is_active() {
                color = theme::color::active(color, env.get(druid::theme::LABEL_COLOR));
            }
            if ctx.is_hot() {
                color = theme::color::hot(color, env.get(druid::theme::LABEL_COLOR));
            }
            let bounds = ctx.size().to_rect();
            ctx.fill(bounds, &color);
        }))
        .on_click(|ctx, data, _| {
            data.selected = true;
            ctx.submit_command(command::CREATE_TAB.with(data.method.method.clone()));
        });

    Flex::row()
        .with_flex_child(label, 1.0)
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
