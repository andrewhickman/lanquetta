use druid::{widget::prelude::*, Point, Rect, WidgetId, WidgetPod};

use super::{TabId, TabLabel, TabLabelState, TabsData, TabsDataChange, CLOSE_TAB};

const MIN_TAB_SIZE: f64 = 150.0;

pub struct TabsHeader {
    children: Vec<WidgetPod<TabLabelState, TabLabel>>,
    id: WidgetId,
}

impl TabsHeader {
    pub fn new() -> Self {
        let id = WidgetId::next();
        TabsHeader {
            children: Vec::new(),
            id,
        }
    }
}

impl<T> Widget<T> for TabsHeader
where
    T: TabsData,
{
    fn event(&mut self, ctx: &mut EventCtx, event: &Event, data: &mut T, env: &Env) {
        if let Event::Command(command) = event {
            if let Some(&tab_id) = command.get(CLOSE_TAB) {
                data.remove(tab_id);
                ctx.set_handled();
                return;
            }
        }

        self.for_each_label_mut(data, |_, label, label_data| {
            label.event(ctx, event, label_data, env)
        })
    }

    fn lifecycle(&mut self, ctx: &mut LifeCycleCtx, event: &LifeCycle, data: &T, env: &Env) {
        if let LifeCycle::WidgetAdded = event {
            let mut index = 0;
            data.for_each(|tab_id, _| {
                self.children
                    .insert(index, WidgetPod::new(TabLabel::new(self.id, tab_id)));
                index += 1;
                ctx.children_changed();
            });
        }

        self.for_each_label(data, |_, label, label_data| {
            label.lifecycle(ctx, event, label_data, env)
        })
    }

    fn update(&mut self, ctx: &mut UpdateCtx, old_data: &T, data: &T, env: &Env) {
        let mut index = 0;
        data.for_each_label_changed(old_data, |id, change| match change {
            TabsDataChange::Added => {
                self.children
                    .insert(index, WidgetPod::new(TabLabel::new(self.id, id)));
                index += 1;
                ctx.children_changed();
            }
            TabsDataChange::Changed(label_data) => {
                if let Some(label) = self.children.get_mut(index) {
                    label.update(ctx, label_data, env);
                } else {
                    tracing::error!("TabHeader out of sync with data");
                }
                index += 1;
            }
            TabsDataChange::Removed => {
                self.children.remove(index);
                ctx.children_changed();
            }
        });
    }

    fn layout(&mut self, ctx: &mut LayoutCtx, bc: &BoxConstraints, data: &T, env: &Env) -> Size {
        bc.debug_check("TabsHeader");

        let mut height = bc.min().height;
        let mut x = 0.0f64;
        let mut paint_rect = Rect::ZERO;

        let mut remaining_max_width = bc.max().width;
        let mut remaining_children = self.children.len();

        self.for_each_label(data, |_, label, label_data| {
            let max_width = remaining_max_width.max(0.0) / remaining_children as f64;
            let label_bc = BoxConstraints::new(
                Size::new(MIN_TAB_SIZE.min(max_width), bc.min().height),
                Size::new(max_width, bc.max().height),
            );

            let child_size = label.layout(ctx, &label_bc, label_data, env);
            label.set_origin(ctx, Point::new(x, 0.0));

            paint_rect = paint_rect.union(label.layout_rect());
            height = height.max(child_size.height);
            x += child_size.width;
            remaining_max_width -= child_size.width;
            remaining_children -= 1;
        });

        let size = bc.constrain(Size::new(x, height));
        ctx.set_paint_insets(paint_rect - Rect::ZERO.with_size(size));
        size
    }

    fn paint(&mut self, ctx: &mut PaintCtx, data: &T, env: &Env) {
        self.for_each_label(data, |_, label, label_data| {
            label.paint(ctx, label_data, env)
        })
    }

    fn id(&self) -> Option<WidgetId> {
        Some(self.id)
    }
}

impl TabsHeader {
    fn for_each_label<F, T>(&mut self, data: &T, mut f: F)
    where
        T: TabsData,
        F: FnMut(TabId, &mut WidgetPod<TabLabelState, TabLabel>, &TabLabelState),
    {
        let mut children = self.children.iter_mut();
        data.for_each_label(|tab_id, label_data| {
            if let Some(label) = children.next() {
                f(tab_id, label, label_data);
            } else {
                tracing::error!("TabHeader out of sync with data");
            }
        })
    }

    fn for_each_label_mut<F, T>(&mut self, data: &mut T, mut f: F)
    where
        T: TabsData,
        F: FnMut(TabId, &mut WidgetPod<TabLabelState, TabLabel>, &mut TabLabelState),
    {
        let mut children = self.children.iter_mut();
        data.for_each_label_mut(|tab_id, label_data| {
            if let Some(label) = children.next() {
                f(tab_id, label, label_data);
            } else {
                tracing::error!("TabHeader out of sync with data");
            }
        })
    }
}
