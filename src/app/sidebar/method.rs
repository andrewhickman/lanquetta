use druid::{
    widget::prelude::*, widget::Label, ArcStr, Data, FontDescriptor, FontFamily, Lens, WidgetExt,
};

use crate::{app::command, protobuf::ProtobufMethod, theme};

#[derive(Debug, Clone, Data, Lens)]
pub(in crate::app) struct State {
    pub selected: bool,
    pub method: MethodState,
}

#[derive(Debug, Clone, Data)]
pub(in crate::app) struct MethodState {
    method: ProtobufMethod,
}

struct Method {
    label: Box<dyn Widget<State>>,
}

pub(in crate::app) fn build() -> impl Widget<State> {
    Method {
        label: Label::raw()
            .with_font(FontDescriptor::new(FontFamily::SANS_SERIF))
            .with_text_size(16.0)
            .padding(theme::GUTTER_SIZE / 4.0)
            .expand_width()
            .lens(MethodState::name())
            .lens(State::method)
            .boxed(),
    }
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

impl Widget<State> for Method {
    fn event(&mut self, ctx: &mut EventCtx, event: &Event, data: &mut State, _: &Env) {
        match event {
            Event::MouseDown(_) => {
                ctx.set_active(true);
                ctx.request_paint();
            }
            Event::MouseUp(_) => {
                if ctx.is_active() {
                    if ctx.is_hot() {
                        data.selected = true;
                        ctx.submit_command(command::SELECT_METHOD.with(data.method.method.clone()));
                    }

                    ctx.set_active(false);
                    ctx.request_paint();
                }
            }
            _ => (),
        }
    }

    fn lifecycle(&mut self, ctx: &mut LifeCycleCtx, event: &LifeCycle, data: &State, env: &Env) {
        if let LifeCycle::HotChanged(_) = event {
            ctx.request_paint();
        }
        self.label.lifecycle(ctx, event, data, env)
    }

    fn update(&mut self, ctx: &mut UpdateCtx, old_data: &State, data: &State, env: &Env) {
        self.label.update(ctx, old_data, data, env);
        if old_data.selected != data.selected {
            ctx.request_paint()
        }
    }

    fn layout(
        &mut self,
        ctx: &mut LayoutCtx,
        bc: &BoxConstraints,
        data: &State,
        env: &Env,
    ) -> druid::Size {
        self.label.layout(ctx, bc, data, env)
    }

    fn paint(&mut self, ctx: &mut PaintCtx, data: &State, env: &Env) {
        let mut color = env.get(theme::SIDEBAR_BACKGROUND);
        if ctx.is_active() || data.selected {
            color = theme::color::active(color, env.get(druid::theme::LABEL_COLOR));
        } else if ctx.is_hot() {
            color = theme::color::hot(color, env.get(druid::theme::LABEL_COLOR));
        }
        let bounds = ctx.size().to_rect();
        ctx.fill(bounds, &color);

        self.label.paint(ctx, data, env)
    }
}
