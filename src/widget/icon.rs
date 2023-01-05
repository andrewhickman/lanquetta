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
    icon!(settings: "M19.43 12.98c.04-.32.07-.64.07-.98 0-.34-.03-.66-.07-.98l2.11-1.65c.19-.15.24-.42.12-.64l-2-3.46c-.09-.16-.26-.25-.44-.25-.06 0-.12.01-.17.03l-2.49 1c-.52-.4-1.08-.73-1.69-.98l-.38-2.65C14.46 2.18 14.25 2 14 2h-4c-.25 0-.46.18-.49.42l-.38 2.65c-.61.25-1.17.59-1.69.98l-2.49-1c-.06-.02-.12-.03-.18-.03-.17 0-.34.09-.43.25l-2 3.46c-.13.22-.07.49.12.64l2.11 1.65c-.04.32-.07.65-.07.98 0 .33.03.66.07.98l-2.11 1.65c-.19.15-.24.42-.12.64l2 3.46c.09.16.26.25.44.25.06 0 .12-.01.17-.03l2.49-1c.52.4 1.08.73 1.69.98l.38 2.65c.03.24.24.42.49.42h4c.25 0 .46-.18.49-.42l.38-2.65c.61-.25 1.17-.59 1.69-.98l2.49 1c.06.02.12.03.18.03.17 0 .34-.09.43-.25l2-3.46c.12-.22.07-.49-.12-.64l-2.11-1.65zm-1.98-1.71c.04.31.05.52.05.73 0 .21-.02.43-.05.73l-.14 1.13.89.7 1.08.84-.7 1.21-1.27-.51-1.04-.42-.9.68c-.43.32-.84.56-1.25.73l-1.06.43-.16 1.13-.2 1.35h-1.4l-.19-1.35-.16-1.13-1.06-.43c-.43-.18-.83-.41-1.23-.71l-.91-.7-1.06.43-1.27.51-.7-1.21 1.08-.84.89-.7-.14-1.13c-.03-.31-.05-.54-.05-.74s.02-.43.05-.73l.14-1.13-.89-.7-1.08-.84.7-1.21 1.27.51 1.04.42.9-.68c.43-.32.84-.56 1.25-.73l1.06-.43.16-1.13.2-1.35h1.39l.19 1.35.16 1.13 1.06.43c.43.18.83.41 1.23.71l.91.7 1.06-.43 1.27-.51.7 1.21-1.07.85-.89.7.14 1.13zM12 8c-2.21 0-4 1.79-4 4s1.79 4 4 4 4-1.79 4-4-1.79-4-4-4zm0 6c-1.1 0-2-.9-2-2s.9-2 2-2 2 .9 2 2-.9 2-2 2z");

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

            ctx.fill(self.path, &self.color.resolve(env))
        });
    }
}
