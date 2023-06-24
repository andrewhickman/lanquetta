use druid::widget::prelude::*;

pub struct Empty;

pub fn empty() -> Empty {
    Empty
}

impl<T> Widget<T> for Empty {
    fn event(&mut self, _: &mut EventCtx, _: &Event, _: &mut T, _: &Env) {}

    fn lifecycle(&mut self, _: &mut LifeCycleCtx, _: &LifeCycle, _: &T, _: &Env) {}

    fn update(&mut self, _: &mut UpdateCtx, _: &T, _: &T, _: &Env) {}

    fn layout(&mut self, _: &mut LayoutCtx, bc: &BoxConstraints, _: &T, _: &Env) -> Size {
        bc.min()
    }

    fn paint(&mut self, _: &mut PaintCtx, _: &T, _: &Env) {}
}
