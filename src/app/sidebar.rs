use std::path::Path;

use anyhow::Result;
use druid::piet::RenderContext;
use druid::{
    BoxConstraints, Data, Env, Event, EventCtx, LayoutCtx, Lens, LifeCycle, LifeCycleCtx, PaintCtx,
    Size, UpdateCtx, Widget,
};

use crate::app::theme;

#[derive(Debug, Default, Clone, Data, Lens)]
pub(in crate::app) struct State {}

pub(in crate::app) fn build() -> impl druid::Widget<State> {
    Sidebar {}
}

struct Sidebar {}

impl State {
    pub fn add_from_path(&mut self, path: &Path) -> Result<()> {
        todo!()
    }
}

impl Widget<State> for Sidebar {
    fn event(&mut self, _ctx: &mut EventCtx, _event: &Event, _data: &mut State, _env: &Env) {}

    fn lifecycle(
        &mut self,
        _ctx: &mut LifeCycleCtx,
        _event: &LifeCycle,
        _data: &State,
        _env: &Env,
    ) {
    }

    fn update(&mut self, ctx: &mut UpdateCtx, _old_data: &State, _data: &State, _env: &Env) {
        ctx.request_paint();
    }

    fn layout(
        &mut self,
        _layout_ctx: &mut LayoutCtx,
        bc: &BoxConstraints,
        _data: &State,
        _env: &Env,
    ) -> Size {
        bc.constrain((400.0, bc.max().height))
    }

    fn paint(&mut self, ctx: &mut PaintCtx, _data: &State, env: &Env) {
        let rect = ctx.size().to_rect();
        ctx.fill(rect, &env.get(theme::SIDEBAR_BACKGROUND_COLOR));
    }
}
