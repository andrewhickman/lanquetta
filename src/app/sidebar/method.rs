use druid::{
    widget::Label, widget::Painter, ArcStr, Data, FontDescriptor, FontFamily, Lens, RenderContext,
    Widget, WidgetExt,
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

pub(in crate::app) fn build() -> Box<dyn Widget<State>> {
    Label::raw()
        .with_font(FontDescriptor::new(FontFamily::SANS_SERIF))
        .with_text_size(16.0)
        .padding((theme::GUTTER_SIZE, theme::GUTTER_SIZE / 4.0))
        .expand_width()
        .lens(MethodState::name())
        .lens(State::method)
        .background(Painter::new(|ctx, data: &State, env| {
            let mut color = env.get(theme::SIDEBAR_BACKGROUND);
            if ctx.is_active() || data.selected {
                color = theme::color::active(color, env.get(druid::theme::LABEL_COLOR));
            } else if ctx.is_hot() {
                color = theme::color::hot(color, env.get(druid::theme::LABEL_COLOR));
            }
            let bounds = ctx.size().to_rect();
            ctx.fill(bounds, &color);
        }))
        .on_click(|ctx, data, _| {
            data.selected = true;
            ctx.submit_command(command::SELECT_METHOD.with(data.method.method.clone()));
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
