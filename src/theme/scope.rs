use druid::{
    BoxConstraints, Env, Event, EventCtx, LayoutCtx, LifeCycle, LifeCycleCtx, PaintCtx, Size,
    UpdateCtx, Widget, WidgetId,
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

    if is_active {
        let color = color::active(color::BOLD_ACCENT);
        env.set(druid::theme::BUTTON_DARK, color.clone());
        env.set(druid::theme::BUTTON_LIGHT, color);
    } else if is_hot {
        let color = color::hot(color::BOLD_ACCENT);
        env.set(druid::theme::BUTTON_DARK, color.clone());
        env.set(druid::theme::BUTTON_LIGHT, color);
    }

    env
}
