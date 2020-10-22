mod address;
mod request;
mod response;

use std::{sync::Arc, collections::BTreeMap};

use druid::{
    lens,
    widget::{prelude::*, Flex, Label, MainAxisAlignment, Painter},
    Color, Data, Lens, LensExt, Point, Rect, Size, Widget, WidgetExt, WidgetPod,
};
use iter_set::Inclusion;

use crate::{app::command, protobuf::ProtobufMethod, theme, widget::Icon};

pub type TabId = u32;

const MIN_TAB_SIZE: f64 = 200.0;

#[derive(Debug, Default, Clone, Data, Lens)]
pub(in crate::app) struct State {
    tabs: im::OrdMap<TabId, Arc<TabState>>,
    current: Option<TabId>,
}

#[derive(Debug, Clone, Data, Lens)]
struct TabLabelState {
    tab: TabState,
    current: bool,
}

#[derive(Debug, Clone, Data, Lens)]
struct TabState {
    method: ProtobufMethod,
    address: address::State,
    request: request::State,
    response: response::State,
}

struct TabsHeader<W, F> {
    labels: BTreeMap<TabId, WidgetPod<TabLabelState, W>>,
    build_label: F,
    id: WidgetId,
}

struct TabsBody<W, F> {
    widgets: BTreeMap<TabId, WidgetPod<TabState, W>>,
    build_widget: F,
    id: WidgetId,
}

pub fn next_tab_id() -> TabId {
    use std::sync::atomic::*;

    static COUNTER: AtomicU32 = AtomicU32::new(0);

    COUNTER.fetch_add(1, Ordering::Relaxed)
}

pub(in crate::app) fn build() -> Box<dyn Widget<State>> {
    Flex::column()
        .with_child(build_tab_header())
        .with_flex_child(build_tabs_body(), 1.0)
        .boxed()
}

fn build_tab_header() -> impl Widget<State> {
    TabsHeader {
        labels: BTreeMap::new(),
        build_label: |widget_id, tab_id| build_tab_label(widget_id, tab_id),
        id: WidgetId::next(),
    }
}

fn build_tab_label(widget_id: WidgetId, tab_id: TabId) -> impl Widget<TabLabelState> {
    Flex::row()
        .main_axis_alignment(MainAxisAlignment::SpaceBetween)
        .with_child(
            Label::raw()
                .lens(lens::Field::new(
                    |tab: &TabState| tab.method.name(),
                    |tab: &mut TabState| tab.method.name_mut(),
                ))
                .lens(TabLabelState::tab)
                .padding(4.0)
                .background(Painter::new(|ctx, data: &TabLabelState, env| {
                    let color = tab_label_background_color(ctx, data, env);
                    let bounds = ctx.size().to_rect();
                    ctx.fill(bounds, &color);
                }))
                .on_click(move |_, data: &mut TabLabelState, _| {
                    data.current = true;
                }),
        )
        .with_child(
            Icon::close()
                .padding(4.0)
                .background(Painter::new(|ctx, data: &TabLabelState, env| {
                    let color = tab_label_cross_background_color(ctx, data, env);
                    let bounds = ctx.size().to_rect();
                    ctx.fill(bounds, &color);
                }))
                .on_click(move |ctx, _, _| {
                    ctx.submit_command(command::CLOSE_TAB.with(tab_id).to(widget_id))
                }),
        )
}

fn build_tabs_body() -> impl Widget<State> {
    TabsBody {
        widgets: BTreeMap::new(),
        build_widget: |widget_id, tab_id| build_tab(widget_id, tab_id),
        id: WidgetId::next(),
    }
}

fn build_tab(widget_id: WidgetId, tab_id: TabId) -> impl Widget<TabState> {
    Flex::column()
        .must_fill_main_axis(true)
        .with_child(address::build().lens(TabState::address))
        .with_spacer(theme::GUTTER_SIZE)
        .with_child(Label::new("Request").align_left())
        .with_spacer(theme::GUTTER_SIZE)
        .with_flex_child(request::build().lens(TabState::request), 0.5)
        .with_spacer(theme::GUTTER_SIZE)
        .with_child(Label::new("Response").align_left())
        .with_spacer(theme::GUTTER_SIZE)
        .with_flex_child(response::build().lens(TabState::response), 0.5)
        .padding(theme::GUTTER_SIZE)
}

impl State {
    pub fn select_method(&mut self, method: ProtobufMethod) {
        if self
            .with_current_tab(|tab_data| tab_data.method.same(&method))
            .unwrap_or(false)
        {
            return;
        }

        for (&id, tab) in &self.tabs {
            if tab.method.same(&method) {
                self.current = Some(id);
            }
        }

        let id = next_tab_id();
        self.current = Some(id);
        self.tabs.insert(
            id,
            Arc::new(TabState {
                request: request::State::new(&method),
                response: response::State::default(),
                address: address::State::default(),
                method,
            }),
        );
    }

    pub fn remove_tab(&mut self, id: TabId) {
        self.tabs.remove(&id);
        self.current = self
            .tabs
            .get_prev(&id)
            .or_else(|| self.tabs.get_next(&id))
            .map(|(&tab_id, _)| tab_id);
    }

    fn tab_lens(id: &TabId) -> impl Lens<State, TabState> + '_ {
        State::tabs
            .then(lens::Index::new(id))
            .then(lens::InArc::new::<TabState, TabState>(lens::Id))
    }

    fn tab_label_lens(id: &TabId) -> impl Lens<State, TabLabelState> + '_ {
        struct TabLabelLens(TabId);

        impl Lens<State, TabLabelState> for TabLabelLens {
            fn with<V, F: FnOnce(&TabLabelState) -> V>(&self, data: &State, f: F) -> V {
                let current = data.current == Some(self.0);

                State::tab_lens(&self.0).with(data, |tab_data| {
                    f(&TabLabelState {
                        tab: tab_data.clone(),
                        current,
                    })
                })
            }

            fn with_mut<V, F: FnOnce(&mut TabLabelState) -> V>(&self, data: &mut State, f: F) -> V {
                let mut current = data.current == Some(self.0);

                let result = State::tab_lens(&self.0).with_mut(data, |tab_data| {
                    let mut tab_label_data = TabLabelState {
                        tab: tab_data.clone(),
                        current,
                    };
                    let result = f(&mut tab_label_data);

                    if !tab_label_data.tab.same(&tab_data) {
                        *tab_data = tab_label_data.tab;
                    }
                    if !tab_label_data.current.same(&current) {
                        current = tab_label_data.current;
                    }

                    result
                });

                if current != (data.current == Some(self.0)) {
                    if current {
                        data.current = Some(self.0);
                    } else {
                        data.current = None;
                    }
                }

                result
            }
        }

        TabLabelLens(*id)
    }

    fn with_current_tab<V>(&self, f: impl FnOnce(&TabState) -> V) -> Option<V> {
        if let Some(id) = self.current {
            Some(State::tab_lens(&id).with(self, f))
        } else {
            None
        }
    }

    fn with_current_tab_mut<V>(&mut self, f: impl FnOnce(&mut TabState) -> V) -> Option<V> {
        if let Some(id) = self.current {
            Some(State::tab_lens(&id).with_mut(self, f))
        } else {
            None
        }
    }
}

impl<W, F> Widget<State> for TabsHeader<W, F>
where
    W: Widget<TabLabelState>,
    F: FnMut(WidgetId, TabId) -> W,
{
    fn event(&mut self, ctx: &mut EventCtx, event: &Event, data: &mut State, env: &Env) {
        if let Event::Command(command) = event {
            if let Some(&tab_id) = command.get(command::CLOSE_TAB) {
                data.remove_tab(tab_id);
                ctx.set_handled();
                return;
            }
        }

        for (id, label) in &mut self.labels {
            State::tab_label_lens(id)
                .with_mut(data, |label_data| label.event(ctx, event, label_data, env))
        }
    }

    fn lifecycle(&mut self, ctx: &mut LifeCycleCtx, event: &LifeCycle, data: &State, env: &Env) {
        if let LifeCycle::WidgetAdded = event {
            debug_assert!(data.tabs.is_empty());
        }

        for (id, label) in &mut self.labels {
            State::tab_label_lens(id).with(data, |label_data| {
                label.lifecycle(ctx, event, label_data, env)
            });
        }
    }

    fn update(&mut self, ctx: &mut UpdateCtx, old_data: &State, data: &State, env: &Env) {
        debug_assert_eq!(old_data.tabs.len(), self.labels.len());

        for inclusion in iter_set::classify_by_key(&old_data.tabs, &data.tabs, |(&key, _)| key) {
            match inclusion {
                Inclusion::Left((id, _)) => {
                    self.labels.remove(id);
                    ctx.children_changed();
                }
                Inclusion::Both(_, (&id, tab_data)) => {
                    let label_data = TabLabelState {
                        tab: (**tab_data).clone(),
                        current: data.current == Some(id),
                    };
                    self.labels
                        .get_mut(&id)
                        .unwrap()
                        .update(ctx, &label_data, env);
                }
                Inclusion::Right((&id, _)) => {
                    self.labels
                        .insert(id, WidgetPod::new((self.build_label)(self.id, id)));
                    ctx.children_changed();
                }
            }
        }

        if old_data.current != data.current && data.current.is_some() {
            ctx.request_layout();
        }

        debug_assert_eq!(data.tabs.len(), self.labels.len());
    }

    fn layout(
        &mut self,
        ctx: &mut LayoutCtx,
        bc: &BoxConstraints,
        data: &State,
        env: &Env,
    ) -> Size {
        let mut height = bc.min().height;
        let mut x = 0.0;
        let mut paint_rect = Rect::ZERO;

        for (id, label) in &mut self.labels {
            State::tab_label_lens(id).with(data, |label_data| {
                let label_bc = BoxConstraints::new(
                    Size::new(MIN_TAB_SIZE, bc.min().height),
                    Size::new(f64::INFINITY, bc.max().height),
                );

                let child_size = label.layout(ctx, &label_bc, label_data, env);
                let rect = Rect::from_origin_size(Point::new(x, 0.0), child_size);
                label.set_layout_rect(ctx, label_data, env, rect);

                paint_rect = paint_rect.union(rect);
                height = height.max(child_size.height);
                x += child_size.width;
            });
        }

        let size = bc.constrain(Size::new(x, height));
        ctx.set_paint_insets(paint_rect - Rect::ZERO.with_size(size));
        size
    }

    fn paint(&mut self, ctx: &mut PaintCtx, data: &State, env: &Env) {
        for (id, label) in &mut self.labels {
            State::tab_label_lens(id).with(data, |label_data| label.paint(ctx, label_data, env));
        }
    }

    fn id(&self) -> Option<WidgetId> {
        Some(self.id)
    }
}

impl<W, F> Widget<State> for TabsBody<W, F>
where
    W: Widget<TabState>,
    F: FnMut(WidgetId, TabId) -> W,
{
    fn event(&mut self, ctx: &mut EventCtx, event: &Event, data: &mut State, env: &Env) {
        if hidden_should_receive_event(event) {
            for (id, child) in &mut self.widgets {
                State::tab_lens(id)
                    .with_mut(data, |tab_data| child.event(ctx, event, tab_data, env));
            }
        } else if let Some(id) = data.current {
            State::tab_lens(&id).with_mut(data, |tab_data| {
                self.widgets
                    .get_mut(&id)
                    .unwrap()
                    .event(ctx, event, tab_data, env)
            });
        }
    }

    fn lifecycle(&mut self, ctx: &mut LifeCycleCtx, event: &LifeCycle, data: &State, env: &Env) {
        if let Some(id) = data.current {
            self.widgets
                .get_mut(&id)
                .unwrap()
                .lifecycle(ctx, event, &data.tabs[&id], env);
        }
    }

    fn update(&mut self, ctx: &mut UpdateCtx, old_data: &State, data: &State, env: &Env) {
        debug_assert_eq!(old_data.tabs.len(), self.widgets.len());

        for inclusion in iter_set::classify_by_key(&old_data.tabs, &data.tabs, |(&key, _)| key) {
            match inclusion {
                Inclusion::Left((id, _)) => {
                    self.widgets.remove(id);
                    ctx.children_changed();
                }
                Inclusion::Both(_, (id, tab_data)) => {
                    self.widgets.get_mut(id).unwrap().update(ctx, tab_data, env);
                }
                Inclusion::Right((&id, _)) => {
                    self.widgets
                        .insert(id, WidgetPod::new((self.build_widget)(self.id, id)));
                    ctx.children_changed();
                }
            }
        }

        debug_assert_eq!(data.tabs.len(), self.widgets.len());
    }

    fn layout(
        &mut self,
        ctx: &mut LayoutCtx,
        bc: &BoxConstraints,
        data: &State,
        env: &Env,
    ) -> Size {
        if let Some(id) = data.current {
            let child = self.widgets.get_mut(&id).unwrap();
            let size = child.layout(ctx, bc, &data.tabs[&id], env);
            child.set_layout_rect(ctx, &data.tabs[&id], env, size.to_rect());
            size
        } else {
            Size::ZERO
        }
    }

    fn paint(&mut self, ctx: &mut PaintCtx, data: &State, env: &Env) {
        if let Some(id) = data.current {
            let bounds = ctx.size().to_rect();
            ctx.fill(bounds, &env.get(theme::TAB_BACKGROUND));

            self.widgets
                .get_mut(&id)
                .unwrap()
                .paint(ctx, &data.tabs[&id], env)
        }
    }
}

fn hidden_should_receive_event(evt: &Event) -> bool {
    match evt {
        Event::WindowConnected
        | Event::WindowSize(_)
        | Event::Timer(_)
        | Event::AnimFrame(_)
        | Event::Command(_)
        | Event::Internal(_) => true,
        Event::MouseDown(_)
        | Event::MouseUp(_)
        | Event::MouseMove(_)
        | Event::Wheel(_)
        | Event::KeyDown(_)
        | Event::KeyUp(_)
        | Event::Paste(_)
        | Event::Zoom(_) => false,
    }
}

fn tab_label_background_color(ctx: &PaintCtx, data: &TabLabelState, env: &Env) -> Color {
    if data.current {
        env.get(theme::TAB_BACKGROUND)
    } else if ctx.is_active() {
        theme::color::active(
            env.get(druid::theme::WINDOW_BACKGROUND_COLOR),
            env.get(druid::theme::LABEL_COLOR),
        )
    } else if ctx.is_hot() {
        theme::color::hot(
            env.get(druid::theme::WINDOW_BACKGROUND_COLOR),
            env.get(druid::theme::LABEL_COLOR),
        )
    } else {
        env.get(druid::theme::WINDOW_BACKGROUND_COLOR)
    }
}

fn tab_label_cross_background_color(ctx: &PaintCtx, data: &TabLabelState, env: &Env) -> Color {
    if data.current {
        if ctx.is_active() {
            theme::color::active(
                env.get(theme::TAB_BACKGROUND),
                env.get(druid::theme::LABEL_COLOR),
            )
        } else if ctx.is_hot() {
            theme::color::hot(
                env.get(theme::TAB_BACKGROUND),
                env.get(druid::theme::LABEL_COLOR),
            )
        } else {
            env.get(theme::TAB_BACKGROUND)
        }
    } else {
        tab_label_background_color(ctx, data, env)
    }
}
