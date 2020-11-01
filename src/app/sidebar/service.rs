use std::sync::Arc;

use druid::{ArcStr, Data, FontDescriptor, FontFamily, Lens, MouseButton, Point, Rect, RenderContext, Widget, WidgetExt as _, WidgetPod, widget::{prelude::*, CrossAxisAlignment, Either, Flex, Label, LineBreaking, List, ListIter}};

use crate::{
    app::sidebar::method,
    protobuf::{ProtobufMethod, ProtobufService},
    theme,
    widget::{Empty, Icon},
};

#[derive(Debug, Clone, Data, Lens)]
pub(in crate::app) struct State {
    pub index: usize,
    pub selected: Option<ProtobufMethod>, // TODO generate from selected tab in body
    pub service: ServiceState,
}

#[derive(Debug, Clone, Data, Lens)]
pub(in crate::app) struct ServiceState {
    name: ArcStr,
    #[lens(ignore)]
    methods: Arc<[method::MethodState]>,
    #[lens(ignore)]
    expanded: bool,
    #[data(ignore)]
    #[lens(ignore)]
    service: ProtobufService,
}

struct Service {
    expanded: WidgetPod<ServiceState, Box<dyn Widget<ServiceState>>>,
    label: WidgetPod<ServiceState, Box<dyn Widget<ServiceState>>>,
    // close: WidgetPod<State, Box<dyn Widget<State>>>,
}

pub(in crate::app) fn build() -> Box<dyn Widget<State>> {
    let service = Service {
        expanded: WidgetPod::new(
            Either::new(
                |state: &ServiceState, _| state.expanded,
                Icon::expand_more(),
                Icon::chevron_right(),
            )
            .boxed(),
        ),
        label: WidgetPod::new(
            Label::raw()
                .with_font(FontDescriptor::new(FontFamily::SANS_SERIF))
                .with_text_size(18.0)
                .with_line_break_mode(LineBreaking::Clip)
                .lens(ServiceState::name)
                .boxed(),
        ),
        // close:
    }.expand_width();

    let methods = Either::new(
        |state: &State, _| state.service.expanded,
        List::new(method::build),
        Empty,
    );

    Flex::column()
        .cross_axis_alignment(CrossAxisAlignment::Start)
        .with_child(service)
        .with_child(methods)
        .boxed()
}

impl State {
    pub fn new(selected: Option<ProtobufMethod>, service: ServiceState) -> Self {
        State { selected, service }
    }

    fn has_selected(&self) -> bool {
        if let Some(selected_method) = &self.selected {
            self.service
                .methods
                .iter()
                .any(|method| selected_method.same(method.method()))
        } else {
            false
        }
    }
}

impl ServiceState {
    pub fn new(service: ProtobufService, expanded: bool) -> Self {
        ServiceState {
            name: service.name().into(),
            methods: service.methods().map(method::MethodState::from).collect(),
            expanded,
            service,
        }
    }

    pub fn service(&self) -> &ProtobufService {
        &self.service
    }

    pub fn expanded(&self) -> bool {
        self.expanded
    }
}

impl From<ProtobufService> for ServiceState {
    fn from(service: ProtobufService) -> Self {
        ServiceState::new(service, true)
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
        for (i, method) in self.service.methods.iter().enumerate() {
            let selected = match &self.selected {
                Some(selected_method) => selected_method.same(method.method()),
                None => false,
            };
            let mut state = method::State::new(selected, method.to_owned());
            cb(&mut state, i);

            debug_assert!(selected.same(&state.selected));
            debug_assert!(method.same(&state.method));
        }
    }

    fn data_len(&self) -> usize {
        self.service.methods.len()
    }
}

impl Widget<State> for Service {
    fn event(&mut self, ctx: &mut EventCtx, event: &Event, data: &mut State, env: &Env) {
        self.expanded.event(ctx, event, &mut data.service, env);
        self.label.event(ctx, event, &mut data.service, env);

        // let close_was_hot = self.close.is_hot();
        // self.close.event(ctx, event, data, env);
        // if self.close.is_hot() != close_was_hot {
        //     ctx.request_paint();
        // }

        if !ctx.is_handled() {
            match event {
                Event::MouseDown(mouse_event) => {
                    if mouse_event.button == MouseButton::Left {
                        ctx.set_active(true);
                        ctx.request_paint();
                    }
                }
                Event::MouseUp(mouse_event) => {
                    if ctx.is_active() && mouse_event.button == MouseButton::Left {
                        ctx.set_active(false);
                        if ctx.is_hot() {
                            data.service.expanded = !data.service.expanded;
                        }
                        ctx.request_paint();
                    }
                }
                _ => {}
            }
        }
    }

    fn lifecycle(&mut self, ctx: &mut LifeCycleCtx, event: &LifeCycle, data: &State, env: &Env) {
        if let LifeCycle::HotChanged(_) = event {
            ctx.request_paint();
        }

        self.expanded.lifecycle(ctx, event, &data.service, env);
        self.label.lifecycle(ctx, event, &data.service, env);
    }

    fn update(&mut self, ctx: &mut UpdateCtx, _: &State, data: &State, env: &Env) {
        self.expanded.update(ctx, &data.service, env);
        self.label.update(ctx, &data.service, env);
    }

    fn layout(
        &mut self,
        ctx: &mut LayoutCtx,
        bc: &BoxConstraints,
        data: &State,
        env: &Env,
    ) -> Size {
        const GUTTER: f64 = 8.0;
        const PADDING: f64 = 8.0;

        let bc = bc.shrink((PADDING * 2.0, PADDING * 2.0));

        let child_bc = BoxConstraints::new(
            Size::new(0.0, bc.min().height),
            Size::new(f64::INFINITY, bc.max().height),
        );

        let expanded_icon_size = self.expanded.layout(ctx, &child_bc, &data.service, env);
        let label_size = self.label.layout(ctx, &child_bc, &data.service, env);

        let total_size = Size::new(
            bc.max().width,
            expanded_icon_size.height.max(label_size.height),
        )
        .clamp(bc.min(), bc.max());

        let expanded_icon_rect = Rect::from_origin_size(
            Point::new(
                PADDING,
                PADDING + (total_size.height - expanded_icon_size.height) / 2.0,
            ),
            expanded_icon_size,
        )
        .expand();
        let label_rect = Rect::from_origin_size(
            Point::new(
                PADDING + expanded_icon_size.width + GUTTER,
                PADDING + (total_size.height - label_size.height) / 2.0,
            ),
            label_size,
        );

        self.expanded.set_layout_rect(ctx, &data.service, env, expanded_icon_rect);
        self.label.set_layout_rect(ctx, &data.service, env, label_rect);

        Size::new(
            PADDING * 2.0 + total_size.width,
            PADDING * 2.0 + total_size.height,
        )
    }

    fn paint(&mut self, ctx: &mut PaintCtx, data: &State, env: &Env) {
        let mut background_color = env.get(theme::SIDEBAR_BACKGROUND);
        if !data.service.expanded && data.has_selected() {
            background_color =
                theme::color::active(background_color, env.get(druid::theme::LABEL_COLOR));
        }
        if ctx.is_active() {
            background_color =
                theme::color::active(background_color, env.get(druid::theme::LABEL_COLOR));
        } else if ctx.is_hot() {
            background_color =
                theme::color::hot(background_color, env.get(druid::theme::LABEL_COLOR));
        }
        let bounds = ctx.size().to_rect();
        ctx.fill(bounds, &background_color);

        self.expanded.paint(ctx, &data.service, env);
        self.label.paint(ctx, &data.service, env);
    }
}
