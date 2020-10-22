use druid::{
    kurbo::BezPath,
    widget::{prelude::*, FillStrat},
    Color, Data, KeyOrValue, Size,
};

const DEFAULT_SIZE: Size = Size::new(24.0, 24.0);

pub struct Icon {
    path: BezPath,
    fill: FillStrat,
    color: KeyOrValue<Color>,
}

impl Icon {
    pub fn chevron_right() -> Self {
        Icon::new("M10 6L8.59 7.41 13.17 12l-4.58 4.59L10 18l6-6z")
    }

    pub fn expand_more() -> Self {
        Icon::new("M16.59 8.59L12 13.17 7.41 8.59 6 10l6 6 6-6z")
    }

    pub fn close() -> Self {
        Icon::new("M19 6.41L17.59 5 12 10.59 6.41 5 5 6.41 10.59 12 5 17.59 6.41 19 12 13.41 17.59 19 19 17.59 13.41 12z")
    }

    fn new(svg_path: &str) -> Self {
        Icon {
            path: BezPath::from_svg(svg_path).unwrap(),
            fill: FillStrat::default(),
            color: druid::theme::LABEL_COLOR.into(),
        }
    }
}

impl<T: Data> Widget<T> for Icon {
    fn event(&mut self, _: &mut EventCtx, _: &Event, _: &mut T, _: &Env) {}

    fn lifecycle(&mut self, _: &mut LifeCycleCtx, _: &LifeCycle, _: &T, _: &Env) {}

    fn update(&mut self, _: &mut UpdateCtx, _: &T, _: &T, _: &Env) {}

    fn layout(&mut self, _: &mut LayoutCtx, bc: &BoxConstraints, _: &T, _: &Env) -> Size {
        bc.constrain(DEFAULT_SIZE)
    }

    fn paint(&mut self, ctx: &mut PaintCtx, _: &T, env: &Env) {
        ctx.with_save(|ctx| {
            if self.fill != FillStrat::Contain {
                let clip_rect = ctx.size().to_rect();
                ctx.clip(clip_rect);
            }

            let offset = self.fill.affine_to_fill(ctx.size(), DEFAULT_SIZE);
            ctx.transform(offset);

            ctx.fill(&self.path, &self.color.resolve(env))
        });
    }
}
