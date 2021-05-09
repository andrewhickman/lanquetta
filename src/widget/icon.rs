use druid::{
    kurbo::BezPath,
    widget::{prelude::*, FillStrat},
    Color, Data, KeyOrValue, Size,
};
use once_cell::sync::Lazy;

const DEFAULT_SIZE: Size = Size::new(24.0, 24.0);

pub struct Icon {
    path: &'static BezPath,
    fill: FillStrat,
    color: KeyOrValue<Color>,
    size: Size,
}

macro_rules! icon {
    ($name:ident: $path:expr) => {
        pub fn $name() -> Self {
            static PATH: Lazy<BezPath> = Lazy::new(|| BezPath::from_svg($path).unwrap());
            Icon::new(&PATH)
        }
    };
}

impl Icon {
    icon!(chevron_right: "M10 6L8.59 7.41 13.17 12l-4.58 4.59L10 18l6-6z");
    icon!(expand_more: "M16.59 8.59L12 13.17 7.41 8.59 6 10l6 6 6-6z");
    icon!(close: "M19 6.41L17.59 5 12 10.59 6.41 5 5 6.41 10.59 12 5 17.59 6.41 19 12 13.41 17.59 19 19 17.59 13.41 12z");
    icon!(add: "M19 13h-6v6h-2v-6H5v-2h6V5h2v6h6v2z");
    icon!(check: "M9 16.17L4.83 12l-1.42 1.41L9 19 21 7l-1.41-1.41z");
    icon!(unary: "M17 4l4 4l-4 4V9h-12V7h12V4zM7 17h12v-2H7v-3l-4 4l4 4V17z");
    icon!(client_streaming: "M17 4l4 4l-4 4V9h-4V7h4V4zM10 7C9.45 7 9 7.45 9 8s0.45 1 1 1s1-0.45 1 -1S10.55 7 10 7zM6 7C5.45 7 5 7.45 5 8s0.45 1 1 1s1-0.45 1 -1S6.55 7 6 7zM7 17h12v-2H7v-3l-4 4l4 4V17z");
    icon!(server_streaming: "M17 4l4 4l-4 4V9h-12V7h12V4zM7 17h4v-2H7v-3l-4 4l4 4V17zM14 17c0.55 0 1-0.45 1 -1c0-0.55 -0.45 -1 -1 -1s-1 0.45-1 1C13 16.55 13.45 17 14 17zM18 17c0.55 0 1-0.45 1 -1c0-0.55 -0.45 -1 -1 -1s-1 0.45-1 1C17 16.55 17.45 17 18 17z");
    icon!(streaming: "M17 4l4 4l-4 4V9h-4V7h4V4zM10 7C9.45 7 9 7.45 9 8s0.45 1 1 1s1-0.45 1 -1S10.55 7 10 7zM6 7C5.45 7 5 7.45 5 8s0.45 1 1 1s1-0.45 1 -1S6.55 7 6 7zM7 17h4v-2H7v-3l-4 4l4 4V17zM14 17c0.55 0 1-0.45 1 -1c0-0.55 -0.45 -1 -1 -1s-1 0.45-1 1C13 16.55 13.45 17 14 17zM18 17c0.55 0 1-0.45 1 -1c0-0.55 -0.45 -1 -1 -1s-1 0.45-1 1C17 16.55 17.45 17 18 17z");
    icon!(copy: "M16 1H4c-1.1 0-2 .9-2 2v14h2V3h12V1zm3 4H8c-1.1 0-2 .9-2 2v14c0 1.1.9 2 2 2h11c1.1 0 2-.9 2-2V7c0-1.1-.9-2-2-2zm0 16H8V7h11v14z");

    fn new(path: &'static BezPath) -> Self {
        Icon {
            path,
            fill: FillStrat::Cover,
            color: druid::theme::TEXT_COLOR.into(),
            size: DEFAULT_SIZE,
        }
    }

    pub fn with_size(mut self, size: impl Into<Size>) -> Self {
        self.size = size.into();
        self
    }

    pub fn with_color(mut self, color: impl Into<KeyOrValue<Color>>) -> Self {
        self.color = color.into();
        self
    }
}

impl<T: Data> Widget<T> for Icon {
    fn event(&mut self, _: &mut EventCtx, _: &Event, _: &mut T, _: &Env) {}

    fn lifecycle(&mut self, _: &mut LifeCycleCtx, _: &LifeCycle, _: &T, _: &Env) {}

    fn update(&mut self, _: &mut UpdateCtx, _: &T, _: &T, _: &Env) {}

    fn layout(&mut self, _: &mut LayoutCtx, bc: &BoxConstraints, _: &T, _: &Env) -> Size {
        bc.debug_check("Icon");

        if bc.is_width_bounded() && bc.is_height_bounded() {
            bc.constrain_aspect_ratio(1.0, self.size.width)
        } else {
            bc.constrain(self.size)
        }
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
