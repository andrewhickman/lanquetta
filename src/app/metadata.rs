use std::sync::Arc;

use druid::{
    widget::{prelude::*, CrossAxisAlignment, Flex, Label, LineBreaking, List, TextBox},
    Lens, Point, WidgetExt, WidgetPod,
};

use crate::{theme::{self, BODY_SPACER}, widget::Icon};

pub type State = Arc<Vec<Entry>>;

#[derive(Debug, Default, Clone, Data, Lens)]
pub struct Entry {
    pub key: String,
    pub value: String,
}

const GRID_NARROW_SPACER: f64 = 2.0;

pub(in crate::app) fn build() -> impl Widget<State> {
    List::new(build_row).with_spacing(GRID_NARROW_SPACER)
}

pub(in crate::app) fn build_editable() -> impl Widget<State> {
    EditableLayout {
        add_button: WidgetPod::new(build_add_button().boxed()),
        metadata: WidgetPod::new(build().scroll().vertical().boxed()),
    }
}

fn build_row() -> impl Widget<Entry> {
    Flex::row()
        .cross_axis_alignment(CrossAxisAlignment::Fill)
        .with_flex_child(
            theme::text_box_scope(TextBox::<String>::new().expand_width()).lens(Entry::key),
            0.33,
        )
        .with_spacer(GRID_NARROW_SPACER)
        .with_flex_child(
            theme::text_box_scope(TextBox::<String>::new().expand_width()).lens(Entry::value),
            0.67,
        )
}

fn build_add_button() -> impl Widget<State> {
    Flex::row()
        .with_child(Icon::add().padding(3.0))
        .with_child(
            Label::new("Add metadata")
                .with_font(theme::font::HEADER_TWO)
                .with_line_break_mode(LineBreaking::Clip),
        )
        .must_fill_main_axis(true)
        .on_click(|_, state: &mut State, _| {
            Arc::make_mut(state).push(Entry::default());
        })
        .background(theme::hot_or_active_painter(
            druid::theme::BUTTON_BORDER_RADIUS,
        ))
}

struct EditableLayout {
    metadata: WidgetPod<State, Box<dyn Widget<State>>>,
    add_button: WidgetPod<State, Box<dyn Widget<State>>>,
}

impl Widget<State> for EditableLayout {
    fn event(&mut self, ctx: &mut EventCtx, event: &Event, data: &mut State, env: &Env) {
        self.metadata.event(ctx, event, data, env);
        self.add_button.event(ctx, event, data, env);
    }

    fn lifecycle(&mut self, ctx: &mut LifeCycleCtx, event: &LifeCycle, data: &State, env: &Env) {
        self.metadata.lifecycle(ctx, event, data, env);
        self.add_button.lifecycle(ctx, event, data, env);
    }

    fn update(&mut self, ctx: &mut UpdateCtx, _: &State, data: &State, env: &Env) {
        self.metadata.update(ctx, data, env);
        self.add_button.update(ctx, data, env);
    }

    fn layout(
        &mut self,
        ctx: &mut LayoutCtx,
        bc: &BoxConstraints,
        data: &State,
        env: &Env,
    ) -> Size {
        let body_spacer = if data.is_empty() {
            0.0
        } else {
            BODY_SPACER
        };

        let width = bc.max().width;
        let max_height = (bc.max().height - GRID_NARROW_SPACER - body_spacer).max(bc.min().height);
        let tight_bc = BoxConstraints::new(
            Size::new(width, bc.min().height),
            Size::new(width, max_height),
        );

        let add_button_size = self.add_button.layout(ctx, &tight_bc, data, env);

        let metadata_bc = tight_bc
            .shrink_max_height_to(bc.max().height - add_button_size.height - GRID_NARROW_SPACER - body_spacer);
        let metadata_size = self.metadata.layout(ctx, &metadata_bc, data, env);

        self.metadata
            .set_origin(ctx, Point::new(0.0, body_spacer));
        self.add_button.set_origin(ctx, Point::new(0.0,  body_spacer + metadata_size.height + GRID_NARROW_SPACER));

        bc.constrain(Size::new(
            width,
            body_spacer + add_button_size.height + GRID_NARROW_SPACER + metadata_size.height,
        ))
    }

    fn paint(&mut self, ctx: &mut PaintCtx, data: &State, env: &Env) {
        self.metadata.paint(ctx, data, env);
        self.add_button.paint(ctx, data, env);
    }
}
