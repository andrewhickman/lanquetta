use druid::{
    BoxConstraints, Color, Env, Event, EventCtx, Key, LayoutCtx, LifeCycle, LifeCycleCtx, PaintCtx,
    Size, UpdateCtx, Widget, WidgetId,
};

use crate::theme::color;

pub fn new<T>(widget: impl Widget<T>) -> impl Widget<T> {
    Scope { widget }
}

struct Scope<W> {
    widget: W,
}

impl<T, W> Widget<T> for Scope<W>
where
    W: Widget<T>,
{
    fn event(&mut self, ctx: &mut EventCtx, event: &Event, data: &mut T, env: &Env) {
        self.widget.event(
            ctx,
            event,
            data,
            &update_env(env, ctx.is_hot(), ctx.is_active()),
        )
    }

    fn lifecycle(&mut self, ctx: &mut LifeCycleCtx, event: &LifeCycle, data: &T, env: &Env) {
        if let LifeCycle::HotChanged(_) = event {
            ctx.request_paint();
        }

        self.widget.lifecycle(
            ctx,
            event,
            data,
            &update_env(env, ctx.is_hot(), ctx.is_active()),
        )
    }

    fn update(&mut self, ctx: &mut UpdateCtx, old_data: &T, data: &T, env: &Env) {
        self.widget.update(
            ctx,
            old_data,
            data,
            &update_env(env, ctx.is_hot(), ctx.is_active()),
        )
    }

    fn layout(&mut self, ctx: &mut LayoutCtx, bc: &BoxConstraints, data: &T, env: &Env) -> Size {
        self.widget.layout(ctx, bc, data, env)
    }

    fn paint(&mut self, ctx: &mut PaintCtx, data: &T, env: &Env) {
        self.widget
            .paint(ctx, data, &update_env(env, ctx.is_hot(), ctx.is_active()))
    }

    fn id(&self) -> Option<WidgetId> {
        self.widget.id()
    }
}

fn update_env(env: &Env, is_hot: bool, is_active: bool) -> Env {
    let mut env = env.clone();

    update_env_key(&mut env, is_hot, is_active, druid::theme::BUTTON_DARK);
    update_env_key(&mut env, is_hot, is_active, druid::theme::BUTTON_LIGHT);

    env
}

fn update_env_key(env: &mut Env, is_hot: bool, is_active: bool, key: Key<Color>) {
    if is_active {
        env.set(
            key.clone(),
            color::active(env.get(key), env.get(druid::theme::LABEL_COLOR)),
        );
    } else if is_hot {
        env.set(
            key.clone(),
            color::hot(env.get(key), env.get(druid::theme::LABEL_COLOR)),
        );
    }
}
