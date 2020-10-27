use druid::{
    BoxConstraints, Color, Env, Event, EventCtx, Key, LayoutCtx, LifeCycle, LifeCycleCtx, PaintCtx,
    Size, UpdateCtx, Widget, WidgetId,
};

use crate::theme::color;

pub struct WidgetState {
    is_hot: bool,
    is_active: bool,
}

pub fn new<T>(
    widget: impl Widget<T>,
    update_env: impl FnMut(&mut Env, &WidgetState),
) -> impl Widget<T> {
    Scope {
        widget,
        update_env,
        env: Env::default(),
    }
}

struct Scope<W, F> {
    widget: W,
    update_env: F,
    env: Env,
}

impl WidgetState {
    pub fn is_hot(&self) -> bool {
        self.is_hot
    }

    pub fn is_active(&self) -> bool {
        self.is_active
    }
}

impl<T, W, F> Widget<T> for Scope<W, F>
where
    W: Widget<T>,
    F: FnMut(&mut Env, &WidgetState),
{
    fn event(&mut self, ctx: &mut EventCtx, event: &Event, data: &mut T, env: &Env) {
        let was_active = ctx.is_active();
        self.widget.event(ctx, event, data, &self.env);
        if ctx.is_active() != was_active {
            self.update_env(env, ctx.is_hot(), ctx.is_active());
            ctx.request_paint();
        }
    }

    fn lifecycle(&mut self, ctx: &mut LifeCycleCtx, event: &LifeCycle, data: &T, env: &Env) {
        if let LifeCycle::WidgetAdded = event {
            self.update_env(env, ctx.is_hot(), ctx.is_active());
        } else if let LifeCycle::HotChanged(_) = event {
            self.update_env(env, ctx.is_hot(), ctx.is_active());
            ctx.request_paint();
        }

        self.widget.lifecycle(ctx, event, data, &self.env)
    }

    fn update(&mut self, ctx: &mut UpdateCtx, old_data: &T, data: &T, env: &Env) {
        if ctx.env_changed() {
            self.update_env(env, ctx.is_hot(), ctx.is_active());
            ctx.request_paint();
        }

        self.widget.update(ctx, old_data, data, &self.env)
    }

    fn layout(&mut self, ctx: &mut LayoutCtx, bc: &BoxConstraints, data: &T, _: &Env) -> Size {
        self.widget.layout(ctx, bc, data, &self.env)
    }

    fn paint(&mut self, ctx: &mut PaintCtx, data: &T, _: &Env) {
        self.widget.paint(ctx, data, &self.env)
    }

    fn id(&self) -> Option<WidgetId> {
        self.widget.id()
    }
}

impl<W, F> Scope<W, F>
where
    F: FnMut(&mut Env, &WidgetState),
{
    fn update_env(&mut self, env: &Env, is_hot: bool, is_active: bool) {
        self.env = env.clone();
        (self.update_env)(&mut self.env, &WidgetState { is_hot, is_active });
    }
}

pub fn set_hot(env: &mut Env, state: &WidgetState, key: Key<Color>) {
    if state.is_hot() {
        env.set(
            key.clone(),
            color::hot(env.get(key), env.get(druid::theme::LABEL_COLOR)),
        );
    }
}

pub fn set_hot_active(env: &mut Env, state: &WidgetState, key: Key<Color>) {
    if state.is_active() {
        env.set(
            key.clone(),
            color::active(env.get(key), env.get(druid::theme::LABEL_COLOR)),
        );
    } else if state.is_hot() {
        env.set(
            key.clone(),
            color::hot(env.get(key), env.get(druid::theme::LABEL_COLOR)),
        );
    }
}
