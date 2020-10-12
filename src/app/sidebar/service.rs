use druid::{
    widget::{prelude::*, CrossAxisAlignment, Either, Flex, Label, LineBreaking, List, ListIter},
    ArcStr, Data, FontDescriptor, FontFamily, Lens, Widget, WidgetExt,
};

use crate::{
    app::sidebar::method,
    protobuf::{ProtobufMethod, ProtobufService},
    theme,
    widget::Empty,
};

#[derive(Debug, Clone, Data, Lens)]
pub(in crate::app) struct State {
    pub selected: Option<ProtobufMethod>,
    pub service: ServiceState,
}

#[derive(Debug, Clone, Data, Lens)]
pub(in crate::app) struct ServiceState {
    name: ArcStr,
    methods: im::Vector<method::MethodState>,
    expanded: bool,
}

struct Service<W> {
    child: W,
}

pub(in crate::app) fn build() -> Box<dyn Widget<State>> {
    let name = Service {
        child: Label::raw()
            .with_font(FontDescriptor::new(FontFamily::SANS_SERIF))
            .with_text_size(18.0)
            .with_line_break_mode(LineBreaking::Clip)
            .padding(theme::GUTTER_SIZE / 2.0)
            .expand_width()
            .lens(ServiceState::name),
    }
    .lens(State::service);
    let methods = Either::new(
        |state: &State, _| state.service.expanded,
        List::new(method::build),
        Empty,
    );

    Flex::column()
        .cross_axis_alignment(CrossAxisAlignment::Start)
        .with_child(name)
        .with_child(methods)
        .boxed()
}

impl State {
    pub fn new(selected: Option<ProtobufMethod>, service: ServiceState) -> Self {
        State { selected, service }
    }
}

impl From<ProtobufService> for ServiceState {
    fn from(service: ProtobufService) -> Self {
        ServiceState {
            name: service.name().into(),
            methods: service.methods().map(method::MethodState::from).collect(),
            expanded: true,
        }
    }
}

impl ListIter<method::State> for State {
    fn for_each(&self, mut cb: impl FnMut(&method::State, usize)) {
        for (i, method) in self.service.methods.iter().enumerate() {
            let selected = match &self.selected {
                Some(selected_method) => selected_method.same(method.method()),
                None => false,
            };
            let state = method::State::new(selected, method.to_owned());
            cb(&state, i);
        }
    }

    fn for_each_mut(&mut self, mut cb: impl FnMut(&mut method::State, usize)) {
        for (i, method) in self.service.methods.iter_mut().enumerate() {
            let selected = match &self.selected {
                Some(selected_method) => selected_method.same(method.method()),
                None => false,
            };
            let mut state = method::State::new(selected, method.to_owned());
            cb(&mut state, i);

            if selected != state.selected {
                self.selected = if state.selected {
                    Some(state.method.method().to_owned())
                } else {
                    None
                };
            }
            if !method.same(&state.method) {
                *method = state.method;
            }
        }
    }

    fn data_len(&self) -> usize {
        self.service.methods.len()
    }
}

impl<W> Widget<ServiceState> for Service<W>
where
    W: Widget<ServiceState>,
{
    fn event(&mut self, ctx: &mut EventCtx, event: &Event, data: &mut ServiceState, _: &Env) {
        match event {
            Event::MouseDown(_) => {
                ctx.set_active(true);
                ctx.request_paint();
            }
            Event::MouseUp(_) => {
                if ctx.is_active() {
                    if ctx.is_hot() {
                        data.expanded = !data.expanded;
                    }

                    ctx.set_active(false);
                    ctx.request_paint();
                }
            }
            _ => (),
        }
    }

    fn lifecycle(
        &mut self,
        ctx: &mut LifeCycleCtx,
        event: &LifeCycle,
        data: &ServiceState,
        env: &Env,
    ) {
        if let LifeCycle::HotChanged(_) = event {
            ctx.request_paint();
        }
        self.child.lifecycle(ctx, event, data, env)
    }

    fn update(&mut self, ctx: &mut UpdateCtx, old_data: &ServiceState, data: &ServiceState, env: &Env) {
        self.child.update(ctx, old_data, data, env);
    }

    fn layout(
        &mut self,
        ctx: &mut LayoutCtx,
        bc: &BoxConstraints,
        data: &ServiceState,
        env: &Env,
    ) -> Size {
        self.child.layout(ctx, bc, data, env)
    }

    fn paint(&mut self, ctx: &mut PaintCtx, data: &ServiceState, env: &Env) {
        let mut color = env.get(theme::SIDEBAR_BACKGROUND);
        if ctx.is_active() {
            color = theme::color::active(color, env.get(druid::theme::LABEL_COLOR));
        }
        if ctx.is_hot() {
            color = theme::color::hot(color, env.get(druid::theme::LABEL_COLOR));
        }
        let bounds = ctx.size().to_rect();
        ctx.fill(bounds, &color);

        self.child.paint(ctx, data, env)
    }
}
