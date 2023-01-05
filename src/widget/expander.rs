use druid::{
    lens,
    widget::{prelude::*, Controller, CrossAxisAlignment, Either, Flex, Painter},
    Data, MouseButton, Point, Vec2, Widget, WidgetExt, WidgetPod,
};

use crate::{
    theme,
    widget::{Empty, Icon},
};

pub trait ExpanderData: Data {
    fn expanded(&self, env: &Env) -> bool;
    fn toggle_expanded(&mut self, env: &Env);
}

struct ExpanderHeader<T> {
    expanded: WidgetPod<T, Box<dyn Widget<T>>>,
    label: WidgetPod<T, Box<dyn Widget<T>>>,
    buttons: Vec<WidgetPod<T, Box<dyn Widget<T>>>>,
}

pub fn new<T>(
    label: impl Widget<T> + 'static,
    child: impl Widget<T> + 'static,
    buttons: impl Iterator<Item = (Icon, Box<dyn FnMut(&mut EventCtx, &mut T, &Env)>)>,
) -> impl Widget<T>
where
    T: ExpanderData,
{
    let header = ExpanderHeader::new(label.boxed(), buttons);

    let child = Either::new(ExpanderData::expanded, child, Empty);

    Flex::column()
        .cross_axis_alignment(CrossAxisAlignment::Start)
        .with_child(header)
        .with_child(child)
        .boxed()
}

impl<T> ExpanderHeader<T>
where
    T: ExpanderData,
{
    fn new(
        label: Box<dyn Widget<T>>,
        buttons: impl Iterator<Item = (Icon, Box<dyn FnMut(&mut EventCtx, &mut T, &Env)>)>,
    ) -> Self {
        let buttons = buttons
            .map(|(icon, on_close)| {
                WidgetPod::new(
                    icon.background(Painter::new(paint_button_background))
                        .lens::<T, _>(lens::Unit::default())
                        .controller(CloseButtonController { on_close })
                        .boxed(),
                )
            })
            .collect();

        ExpanderHeader {
            expanded: WidgetPod::new(
                Either::new(
                    ExpanderData::expanded,
                    Icon::expand_more(),
                    Icon::chevron_right(),
                )
                .boxed(),
            ),
            label: WidgetPod::new(label),
            buttons,
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

        for button in &mut self.buttons {
            let was_hot = button.is_hot();
            button.event(ctx, event, data, env);
            if button.is_hot() != was_hot {
                ctx.request_paint();
            }
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

        for button in &mut self.buttons {
            button.lifecycle(ctx, event, data, env);
        }
    }

    fn update(&mut self, ctx: &mut UpdateCtx, _: &T, data: &T, env: &Env) {
        self.expanded.update(ctx, data, env);
        self.label.update(ctx, data, env);

        for button in &mut self.buttons {
            button.update(ctx, data, env);
        }
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

        let button_sizes: Vec<_> = self
            .buttons
            .iter_mut()
            .map(|button| button.layout(ctx, &icon_bc, data, env))
            .collect();
        let total_button_width: f64 = button_sizes.iter().map(|sz| padding + sz.width).sum();

        let label_bc =
            inner_bc.shrink((expanded_icon_size.width + padding + total_button_width, 0.0));
        let label_size = self.label.layout(ctx, &label_bc, data, env);

        let total_size = Size::new(
            inner_bc.max().width,
            expanded_icon_size
                .height
                .max(label_size.height)
                .max(button_sizes.iter().map(|sz| sz.height).fold(0.0, f64::max)),
        )
        .clamp(inner_bc.min(), inner_bc.max());

        self.expanded.set_origin(
            ctx,
            origin + Vec2::new(0.0, (total_size.height - expanded_icon_size.height) / 2.0),
        );
        self.label.set_origin(
            ctx,
            origin
                + Vec2::new(
                    expanded_icon_size.width + padding,
                    (total_size.height - label_size.height) / 2.0,
                ),
        );

        let mut button_origin_x = origin.x + total_size.width - total_button_width;
        for (button, sz) in self.buttons.iter_mut().zip(&button_sizes) {
            button_origin_x += padding;
            button.set_origin(
                ctx,
                Point::new(
                    button_origin_x,
                    origin.y + (total_size.height - sz.height) / 2.0,
                ),
            );
            button_origin_x += sz.width;
        }

        padding_size + total_size
    }

    fn paint(&mut self, ctx: &mut PaintCtx, data: &T, env: &Env) {
        let mut bg_color = env.get(theme::EXPANDER_BACKGROUND);
        if ctx.is_active() {
            bg_color = theme::color::active(bg_color, env.get(druid::theme::TEXT_COLOR));
        } else if ctx.is_hot() && self.buttons.iter().all(|b| !b.is_hot()) {
            bg_color = theme::color::hot(bg_color, env.get(druid::theme::TEXT_COLOR));
        }
        let bounds = ctx
            .size()
            .to_rounded_rect(env.get(theme::EXPANDER_CORNER_RADIUS));
        ctx.fill(bounds, &bg_color);

        self.expanded.paint(ctx, data, env);
        self.label.paint(ctx, data, env);

        for button in &mut self.buttons {
            button.paint(ctx, data, env);
        }
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

fn paint_button_background<T>(ctx: &mut PaintCtx, _: &T, env: &Env) {
    if !ctx.is_active() && !ctx.is_hot() {
        return;
    }

    let mut color = env.get(theme::EXPANDER_BACKGROUND);
    if ctx.is_active() {
        color = theme::color::active(color, env.get(druid::theme::TEXT_COLOR));
    } else if ctx.is_hot() {
        color = theme::color::hot(color, env.get(druid::theme::TEXT_COLOR));
    };

    let bounds = ctx
        .size()
        .to_rounded_rect(env.get(druid::theme::BUTTON_BORDER_RADIUS));
    ctx.fill(bounds, &color);
}
