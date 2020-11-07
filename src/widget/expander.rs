use druid::{
    lens,
    widget::{
        prelude::*, Controller, CrossAxisAlignment, Either, Flex, Label, LineBreaking, Painter,
    },
    ArcStr, Data, Lens, MouseButton, Rect, Vec2, Widget, WidgetExt, WidgetPod,
};

use crate::{
    theme,
    widget::{Empty, Icon},
};

pub trait ExpanderData: Data {
    fn expanded(&self, env: &Env) -> bool;
    fn toggle_expanded(&mut self, env: &Env);

    fn with_label<V>(&self, f: impl FnOnce(&ArcStr) -> V) -> V;
    fn with_label_mut<V>(&mut self, f: impl FnOnce(&mut ArcStr) -> V) -> V;

    fn can_close(&self) -> bool;
}

pub struct Expander;

struct ExpanderHeader<T> {
    expanded: WidgetPod<T, Box<dyn Widget<T>>>,
    label: WidgetPod<T, Box<dyn Widget<T>>>,
    close: WidgetPod<T, Box<dyn Widget<T>>>,
}

impl Expander {
    pub fn new<T>(
        on_close: impl FnMut(&mut EventCtx, &mut T, &Env) + 'static,
        child: impl Widget<T> + 'static,
    ) -> impl Widget<T>
    where
        T: ExpanderData,
    {
        let header = ExpanderHeader::new(on_close);

        let child = Either::new(ExpanderData::expanded, child, Empty);

        Flex::column()
            .cross_axis_alignment(CrossAxisAlignment::Start)
            .with_child(header)
            .with_child(child)
            .boxed()
    }
}

impl<T> ExpanderHeader<T>
where
    T: ExpanderData,
{
    fn new(on_close: impl FnMut(&mut EventCtx, &mut T, &Env) + 'static) -> Self {
        ExpanderHeader {
            expanded: WidgetPod::new(
                Either::new(
                    ExpanderData::expanded,
                    Icon::expand_more(),
                    Icon::chevron_right(),
                )
                .boxed(),
            ),
            label: WidgetPod::new(
                Label::raw()
                    .with_font(theme::EXPANDER_LABEL_FONT)
                    .with_line_break_mode(LineBreaking::Clip)
                    .lens::<T, _>(ExpanderLabelLens)
                    .boxed(),
            ),
            close: WidgetPod::new(
                Icon::close()
                    .background(Painter::new(paint_close_background))
                    .lens(lens::Unit::<T>::default())
                    .controller(CloseButtonController {
                        on_close: Box::new(on_close),
                    })
                    .boxed(),
            ),
        }
    }
}

impl<T> Widget<T> for ExpanderHeader<T>
where
    T: ExpanderData,
{
    fn event(&mut self, ctx: &mut EventCtx, event: &Event, data: &mut T, env: &Env) {
        self.expanded.event(ctx, event, data, env);
        self.label.event(ctx, event, data, env);

        let close_was_hot = self.close.is_hot();
        self.close.event(ctx, event, data, env);
        if self.close.is_hot() != close_was_hot {
            ctx.request_paint();
        }

        if !ctx.is_handled() {
            match event {
                Event::MouseDown(mouse_event) => {
                    if mouse_event.button == MouseButton::Left {
                        ctx.set_active(true);
                        ctx.request_paint();
                    }
                }
                Event::MouseUp(mouse_event) => {
                    if ctx.is_active() && mouse_event.button == MouseButton::Left {
                        ctx.set_active(false);
                        if ctx.is_hot() {
                            data.toggle_expanded(env);
                        }
                        ctx.request_paint();
                    }
                }
                _ => {}
            }
        }
    }

    fn lifecycle(&mut self, ctx: &mut LifeCycleCtx, event: &LifeCycle, data: &T, env: &Env) {
        if let LifeCycle::HotChanged(_) = event {
            ctx.request_paint();
        }

        self.expanded.lifecycle(ctx, event, data, env);
        self.label.lifecycle(ctx, event, data, env);
        self.close.lifecycle(ctx, event, data, env);
    }

    fn update(&mut self, ctx: &mut UpdateCtx, _: &T, data: &T, env: &Env) {
        self.expanded.update(ctx, data, env);
        self.label.update(ctx, data, env);
        self.close.update(ctx, data, env);
    }

    fn layout(&mut self, ctx: &mut LayoutCtx, bc: &BoxConstraints, data: &T, env: &Env) -> Size {
        bc.debug_check("Expander");

        let padding = env.get(theme::EXPANDER_PADDING);

        let padding_size = Size::new(padding * 2.0, padding * 2.0).clamp(Size::ZERO, bc.max());
        let inner_bc = bc.shrink(padding_size);
        let origin = (padding_size / 2.0).to_vec2().to_point();

        let icon_bc = BoxConstraints::new(
            Size::new(0.0, inner_bc.min().height),
            Size::new(f64::INFINITY, inner_bc.max().height),
        );

        let expanded_icon_size = self.expanded.layout(ctx, &icon_bc, data, env);
        let close_size = if data.can_close() {
            self.close.layout(ctx, &icon_bc, data, env)
        } else {
            Size::ZERO
        };

        let label_bc = inner_bc.shrink((
            expanded_icon_size.width + padding + padding + close_size.width,
            0.0,
        ));
        let label_size = self.label.layout(ctx, &label_bc, data, env);

        let total_size = Size::new(
            inner_bc.max().width,
            expanded_icon_size
                .height
                .max(label_size.height)
                .max(close_size.height),
        )
        .clamp(inner_bc.min(), inner_bc.max());

        let expanded_icon_rect = Rect::from_origin_size(
            origin + Vec2::new(0.0, (total_size.height - expanded_icon_size.height) / 2.0),
            expanded_icon_size,
        )
        .expand();
        let label_rect = Rect::from_origin_size(
            origin
                + Vec2::new(
                    expanded_icon_size.width + padding,
                    (total_size.height - label_size.height) / 2.0,
                ),
            label_size,
        )
        .expand();
        let close_rect = Rect::from_origin_size(
            origin
                + Vec2::new(
                    total_size.width - close_size.width,
                    (total_size.height - close_size.height) / 2.0,
                ),
            close_size,
        )
        .expand();

        self.expanded
            .set_layout_rect(ctx, data, env, expanded_icon_rect);
        self.label.set_layout_rect(ctx, data, env, label_rect);
        self.close.set_layout_rect(ctx, data, env, close_rect);

        padding_size + total_size
    }

    fn paint(&mut self, ctx: &mut PaintCtx, data: &T, env: &Env) {
        let mut bg_color = env.get(theme::EXPANDER_BACKGROUND);
        if ctx.is_active() {
            bg_color = theme::color::active(bg_color, env.get(druid::theme::LABEL_COLOR));
        } else if ctx.is_hot() && !self.close.is_hot() {
            bg_color = theme::color::hot(bg_color, env.get(druid::theme::LABEL_COLOR));
        }
        let bounds = ctx.size().to_rect();
        ctx.fill(bounds, &bg_color);

        self.expanded.paint(ctx, data, env);
        self.label.paint(ctx, data, env);
        self.close.paint(ctx, data, env);
    }
}

struct ExpanderLabelLens;

impl<T> Lens<T, ArcStr> for ExpanderLabelLens
where
    T: ExpanderData,
{
    fn with<V, F: FnOnce(&ArcStr) -> V>(&self, data: &T, f: F) -> V {
        data.with_label(f)
    }

    fn with_mut<V, F: FnOnce(&mut ArcStr) -> V>(&self, data: &mut T, f: F) -> V {
        data.with_label_mut(f)
    }
}

struct CloseButtonController<T> {
    on_close: Box<dyn FnMut(&mut EventCtx, &mut T, &Env)>,
}

impl<T, W> Controller<T, W> for CloseButtonController<T>
where
    T: ExpanderData,
    W: Widget<T>,
{
    fn event(&mut self, child: &mut W, ctx: &mut EventCtx, event: &Event, data: &mut T, env: &Env) {
        match event {
            Event::MouseDown(mouse_event) => {
                if mouse_event.button == MouseButton::Left {
                    ctx.set_active(true);
                    ctx.request_paint();
                    ctx.set_handled();
                }
            }
            Event::MouseUp(mouse_event) => {
                if ctx.is_active() && mouse_event.button == MouseButton::Left {
                    ctx.set_active(false);
                    if ctx.is_hot() {
                        (self.on_close)(ctx, data, env);
                    }
                    ctx.request_paint();
                    ctx.set_handled();
                }
            }
            _ => {}
        }

        child.event(ctx, event, data, env);
    }

    fn lifecycle(
        &mut self,
        child: &mut W,
        ctx: &mut LifeCycleCtx,
        event: &LifeCycle,
        data: &T,
        env: &Env,
    ) {
        if let LifeCycle::HotChanged(_) = event {
            ctx.request_paint();
        }

        child.lifecycle(ctx, event, data, env);
    }
}

fn paint_close_background<T>(ctx: &mut PaintCtx, _: &T, env: &Env) {
    if !ctx.is_active() && !ctx.is_hot() {
        return;
    }

    let mut color = env.get(theme::EXPANDER_BACKGROUND);
    if ctx.is_active() {
        color = theme::color::active(color, env.get(druid::theme::LABEL_COLOR));
    } else if ctx.is_hot() {
        color = theme::color::hot(color, env.get(druid::theme::LABEL_COLOR));
    };

    let bounds = ctx
        .size()
        .to_rounded_rect(env.get(druid::theme::BUTTON_BORDER_RADIUS));
    ctx.fill(bounds, &color);
}
