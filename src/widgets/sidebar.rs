use druid::piet::RenderContext;
use druid::Widget;

use crate::data::AppState;
use crate::theme;

pub struct Sidebar;

impl Widget<AppState> for Sidebar {
    fn event(
        &mut self,
        _ctx: &mut druid::EventCtx,
        _event: &druid::Event,
        _data: &mut AppState,
        _env: &druid::Env,
    ) {
    }

    fn lifecycle(
        &mut self,
        _ctx: &mut druid::LifeCycleCtx,
        _event: &druid::LifeCycle,
        _data: &AppState,
        _env: &druid::Env,
    ) {
    }

    fn update(
        &mut self,
        _ctx: &mut druid::UpdateCtx,
        _old_data: &AppState,
        _data: &AppState,
        _env: &druid::Env,
    ) {
    }

    fn layout(
        &mut self,
        _ctx: &mut druid::LayoutCtx,
        bc: &druid::BoxConstraints,
        _data: &AppState,
        _env: &druid::Env,
    ) -> druid::Size {
        bc.max()
    }

    fn paint(&mut self, ctx: &mut druid::PaintCtx, _data: &AppState, env: &druid::Env) {
        let rect = ctx.size().to_rect();
        ctx.fill(rect, &env.get(theme::SIDEBAR_BACKGROUND));
    }
}
