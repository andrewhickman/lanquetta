use druid::{
    widget::Controller,
    widget::LineBreaking,
    widget::Painter,
    widget::{prelude::*, RawLabel},
    ArcStr, Data, Lens, MouseButton, Point, Widget, WidgetExt as _, WidgetId, WidgetPod,
};

use super::{TabId, CLOSE_TAB};
use crate::{theme, widget::Icon};

#[derive(Debug, Clone, Data, Lens)]
pub struct State {
    name: ArcStr,
    #[lens(ignore)]
    selected: bool,
}

pub struct TabLabel {
    label: WidgetPod<State, Box<dyn Widget<State>>>,
    close: WidgetPod<State, Box<dyn Widget<State>>>,
    show_close: bool,
}

struct CloseButtonController {
    tabs_id: WidgetId,
    tab_id: TabId,
}

impl TabLabel {
    pub fn new(tabs_id: WidgetId, tab_id: TabId) -> Self {
        TabLabel {
            label: WidgetPod::new(
                RawLabel::new()
                    .with_font(theme::TAB_LABEL_FONT)
                    .with_line_break_mode(LineBreaking::Clip)
                    .lens(State::name)
                    .boxed(),
            ),
            close: WidgetPod::new(
                Icon::close()
                    .fix_size(20.0, 20.0)
                    .background(Painter::new(paint_close_background))
                    .controller(CloseButtonController { tabs_id, tab_id }),
            )
            .boxed(),
            show_close: false,
        }
    }
}

impl State {
    pub fn new(name: ArcStr, selected: bool) -> Self {
        State { name, selected }
    }

    pub fn selected(&self) -> bool {
        self.selected
    }
}

impl Widget<State> for TabLabel {
    fn event(&mut self, ctx: &mut EventCtx, event: &Event, data: &mut State, env: &Env) {
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
                            data.selected = true;
                        }
                        ctx.request_paint();
                    }
                }
                _ => {}
            }
        }
    }

    fn lifecycle(&mut self, ctx: &mut LifeCycleCtx, event: &LifeCycle, data: &State, env: &Env) {
        match event {
            LifeCycle::WidgetAdded => {
                self.show_close = ctx.is_hot() || data.selected;
                ctx.request_layout();
            }
            LifeCycle::HotChanged(_) => {
                if !data.selected {
                    self.show_close = ctx.is_hot();
                    ctx.request_layout();
                }
            }
            _ => (),
        }

        self.label.lifecycle(ctx, event, data, env);
        self.close.lifecycle(ctx, event, data, env);
    }

    fn update(&mut self, ctx: &mut UpdateCtx, old_data: &State, data: &State, env: &Env) {
        if !old_data.selected.same(&data.selected) {
            self.show_close = ctx.is_hot() || data.selected;
            ctx.request_layout();
        }

        self.label.update(ctx, data, env);
        self.close.update(ctx, data, env);
    }

    fn layout(
        &mut self,
        ctx: &mut LayoutCtx,
        bc: &BoxConstraints,
        data: &State,
        env: &Env,
    ) -> Size {
        bc.debug_check("TabLabel");

        const PADDING: f64 = 3.0;

        let bc = bc.shrink((PADDING * 2.0, PADDING * 2.0));

        let close_size = if self.show_close {
            self.close.layout(ctx, &bc.loosen(), data, env)
        } else {
            Size::ZERO
        };
        let label_size = self
            .label
            .layout(ctx, &bc.shrink((close_size.width, 0.0)), data, env);

        let total_size = Size::new(
            label_size.width + close_size.width,
            label_size.height.max(close_size.height),
        )
        .clamp(bc.min(), bc.max());

        self.label.set_origin(
            ctx,
            Point::new(
                PADDING,
                PADDING + (total_size.height - label_size.height) / 2.0,
            ),
        );
        self.close.set_origin(
            ctx,
            Point::new(
                PADDING + total_size.width - close_size.width,
                PADDING + (total_size.height - close_size.height) / 2.0,
            ),
        );

        Size::new(
            PADDING * 2.0 + total_size.width,
            PADDING * 2.0 + total_size.height,
        )
    }

    fn paint(&mut self, ctx: &mut PaintCtx, data: &State, env: &Env) {
        let background_color = if data.selected {
            env.get(theme::SELECTED_TAB_BACKGROUND)
        } else {
            let mut color = env.get(druid::theme::WINDOW_BACKGROUND_COLOR);
            if ctx.is_active() {
                color = theme::color::active(color, env.get(druid::theme::TEXT_COLOR));
            } else if ctx.is_hot() && !self.close.is_hot() {
                color = theme::color::hot(color, env.get(druid::theme::TEXT_COLOR));
            }
            color
        };

        let bounds = ctx.size().to_rect();
        ctx.fill(bounds, &background_color);

        self.label.paint(ctx, data, env);
        if self.show_close {
            self.close.paint(ctx, data, env);
        }
    }
}

impl<W> Controller<State, W> for CloseButtonController
where
    W: Widget<State>,
{
    fn event(
        &mut self,
        child: &mut W,
        ctx: &mut EventCtx,
        event: &Event,
        data: &mut State,
        env: &Env,
    ) {
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
                        ctx.submit_command(CLOSE_TAB.with(self.tab_id).to(self.tabs_id))
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
        data: &State,
        env: &Env,
    ) {
        if let LifeCycle::HotChanged(_) = event {
            ctx.request_paint();
        }

        child.lifecycle(ctx, event, data, env);
    }
}

fn paint_close_background(ctx: &mut PaintCtx, data: &State, env: &Env) {
    if !ctx.is_active() && !ctx.is_hot() {
        return;
    }

    let mut color = if data.selected {
        env.get(theme::SELECTED_TAB_BACKGROUND)
    } else {
        env.get(druid::theme::WINDOW_BACKGROUND_COLOR)
    };

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
