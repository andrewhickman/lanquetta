use std::sync::Arc;

use druid::{
    widget::Controller,
    widget::Painter,
    widget::{prelude::*, CrossAxisAlignment, Either, Flex, Label, LineBreaking, List, ListIter},
    ArcStr, Data, FontDescriptor, FontFamily, Lens, MouseButton, Rect, RenderContext, Vec2, Widget,
    WidgetExt as _, WidgetPod,
};

use crate::{
    app::{command::REMOVE_SERVICE, sidebar::method},
    protobuf::{ProtobufMethod, ProtobufService},
    theme,
    widget::{Empty, Icon},
};

#[derive(Debug, Clone, Data, Lens)]
pub(in crate::app) struct State {
    pub index: usize,
    pub selected: Option<ProtobufMethod>,
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
    close: WidgetPod<State, Box<dyn Widget<State>>>,
}

struct CloseButtonController {
    sidebar_id: WidgetId,
}

pub(in crate::app) fn build(sidebar_id: WidgetId) -> Box<dyn Widget<State>> {
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
        close: WidgetPod::new(
            Icon::close()
                .background(Painter::new(paint_close_background))
                .controller(CloseButtonController { sidebar_id })
                .boxed(),
        ),
    }
    .expand_width();

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
    pub fn new(selected: Option<ProtobufMethod>, service: ServiceState, index: usize) -> Self {
        State {
            selected,
            service,
            index,
        }
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

        let close_was_hot = self.close.is_hot();
        self.close.event(ctx, event, data, env);
        if self.close.is_hot() != close_was_hot {
            ctx.request_paint();
        }

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
        self.close.lifecycle(ctx, event, data, env);
    }

    fn update(&mut self, ctx: &mut UpdateCtx, _: &State, data: &State, env: &Env) {
        self.expanded.update(ctx, &data.service, env);
        self.label.update(ctx, &data.service, env);
        self.close.update(ctx, data, env);
    }

    fn layout(
        &mut self,
        ctx: &mut LayoutCtx,
        bc: &BoxConstraints,
        data: &State,
        env: &Env,
    ) -> Size {
        bc.debug_check("Service");

        const GUTTER: f64 = 8.0;
        const PADDING: f64 = 8.0;

        let padding = Size::new(PADDING * 2.0, PADDING * 2.0).clamp(Size::ZERO, bc.max());
        let inner_bc = bc.shrink(padding);
        let origin = (padding / 2.0).to_vec2().to_point();

        let icon_bc = BoxConstraints::new(
            Size::new(0.0, inner_bc.min().height),
            Size::new(inner_bc.max().width / 2.0, inner_bc.max().height),
        );

        let expanded_icon_size = self.expanded.layout(ctx, &icon_bc, &data.service, env);
        let close_size = self.close.layout(ctx, &icon_bc, data, env);

        let label_bc = inner_bc.shrink((
            expanded_icon_size.width + GUTTER + GUTTER + close_size.width,
            0.0,
        ));
        let label_size = self.label.layout(ctx, &label_bc, &data.service, env);

        let total_size = Size::new(
            inner_bc.max().width,
            expanded_icon_size
                .height
                .max(label_size.height)
                .max(close_size.height),
        )
        .clamp(inner_bc.min(), inner_bc.max());

        let expanded_icon_rect = Rect::from_origin_size(
            origin + Vec2::new(0.0, (total_size.height - expanded_icon_size.height) / 2.0),
            expanded_icon_size,
        )
        .expand();
        let label_rect = Rect::from_origin_size(
            origin
                + Vec2::new(
                    expanded_icon_size.width + GUTTER,
                    (total_size.height - label_size.height) / 2.0,
                ),
            label_size,
        )
        .expand();
        let close_rect = Rect::from_origin_size(
            origin
                + Vec2::new(
                    total_size.width - close_size.width,
                    (total_size.height - close_size.height) / 2.0,
                ),
            close_size,
        )
        .expand();

        self.expanded
            .set_layout_rect(ctx, &data.service, env, expanded_icon_rect);
        self.label
            .set_layout_rect(ctx, &data.service, env, label_rect);
        self.close.set_layout_rect(ctx, data, env, close_rect);

        padding + total_size
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
        } else if ctx.is_hot() && !self.close.is_hot() {
            background_color =
                theme::color::hot(background_color, env.get(druid::theme::LABEL_COLOR));
        }
        let bounds = ctx.size().to_rect();
        ctx.fill(bounds, &background_color);

        self.expanded.paint(ctx, &data.service, env);
        self.label.paint(ctx, &data.service, env);
        self.close.paint(ctx, data, env);
    }
}

impl<W> Controller<State, W> for CloseButtonController
where
    W: Widget<State>,
{
    fn event(
        &mut self,
        child: &mut W,
        ctx: &mut EventCtx,
        event: &Event,
        data: &mut State,
        env: &Env,
    ) {
        match event {
            Event::MouseDown(mouse_event) => {
                if mouse_event.button == MouseButton::Left {
                    ctx.set_active(true);
                    ctx.request_paint();
                    ctx.set_handled();
                }
            }
            Event::MouseUp(mouse_event) => {
                if ctx.is_active() && mouse_event.button == MouseButton::Left {
                    ctx.set_active(false);
                    if ctx.is_hot() {
                        ctx.submit_command(REMOVE_SERVICE.with(data.index).to(self.sidebar_id))
                    }
                    ctx.request_paint();
                    ctx.set_handled();
                }
            }
            _ => {}
        }

        child.event(ctx, event, data, env);
    }

    fn lifecycle(
        &mut self,
        child: &mut W,
        ctx: &mut LifeCycleCtx,
        event: &LifeCycle,
        data: &State,
        env: &Env,
    ) {
        if let LifeCycle::HotChanged(_) = event {
            ctx.request_paint();
        }

        child.lifecycle(ctx, event, data, env);
    }
}

fn paint_close_background(ctx: &mut PaintCtx, data: &State, env: &Env) {
    if !ctx.is_active() && !ctx.is_hot() {
        return;
    }

    let mut color = env.get(theme::SIDEBAR_BACKGROUND);
    if !data.service.expanded && data.has_selected() {
        color = theme::color::active(color, env.get(druid::theme::LABEL_COLOR));
    }
    if ctx.is_active() {
        color = theme::color::active(color, env.get(druid::theme::LABEL_COLOR));
    } else if ctx.is_hot() {
        color = theme::color::hot(color, env.get(druid::theme::LABEL_COLOR));
    };

    let bounds = ctx
        .size()
        .to_rounded_rect(env.get(druid::theme::BUTTON_BORDER_RADIUS));
    ctx.fill(bounds, &color);
}
