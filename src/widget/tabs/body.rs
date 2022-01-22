use std::collections::BTreeMap;

use druid::{widget::prelude::*, Point, Widget, WidgetPod};

use super::{TabId, TabsData, TabsDataChange};
use crate::theme;

pub struct TabsBody<T: TabsData, F, W> {
    children: BTreeMap<TabId, WidgetPod<T::Item, W>>,
    make_child: F,
}

impl<T: TabsData, F, W> TabsBody<T, F, W> {
    pub fn new(make_child: F) -> Self {
        TabsBody {
            children: BTreeMap::new(),
            make_child,
        }
    }
}

impl<T, F, W> Widget<T> for TabsBody<T, F, W>
where
    T: TabsData,
    F: FnMut() -> W,
    W: Widget<T::Item>,
{
    fn event(&mut self, ctx: &mut EventCtx, event: &Event, data: &mut T, env: &Env) {
        let should_propagate_to_hidden = match event {
            Event::Command(command) => data.route_command_to_hidden(command),
            event => event.should_propagate_to_hidden(),
        };

        if should_propagate_to_hidden {
            self.for_each_mut(data, |_, tab, tab_data| {
                tab.event(ctx, event, tab_data, env)
            })
        } else {
            data.with_selected_mut(|id, tab_data| {
                self.children
                    .get_mut(&id)
                    .unwrap()
                    .event(ctx, event, tab_data, env)
            });
        }
    }

    fn lifecycle(&mut self, ctx: &mut LifeCycleCtx, event: &LifeCycle, data: &T, env: &Env) {
        if let LifeCycle::WidgetAdded = event {
            data.for_each(|tab_id, _| {
                self.children
                    .insert(tab_id, WidgetPod::new((self.make_child)()));
                ctx.children_changed();
            });
        }

        if event.should_propagate_to_hidden() {
            self.for_each(data, |_, tab, tab_data| {
                tab.lifecycle(ctx, event, tab_data, env)
            })
        } else {
            data.with_selected(|id, tab_data| {
                self.children
                    .get_mut(&id)
                    .unwrap()
                    .lifecycle(ctx, event, tab_data, env)
            });
        }
    }

    fn update(&mut self, ctx: &mut UpdateCtx, old_data: &T, data: &T, env: &Env) {
        data.for_each_changed(old_data, |id, change| match change {
            TabsDataChange::Added => {
                self.children
                    .insert(id, WidgetPod::new((self.make_child)()));
                ctx.children_changed();
            }
            TabsDataChange::Changed(label_data) => {
                if let Some(label) = self.children.get_mut(&id) {
                    label.update(ctx, label_data, env);
                } else {
                    tracing::error!("TabBody out of sync with data");
                }
            }
            TabsDataChange::Removed => {
                self.children.remove(&id);
                ctx.children_changed();
            }
        });

        if old_data.selected() != data.selected() && data.selected().is_some() {
            ctx.request_layout();
        }
    }

    fn layout(&mut self, ctx: &mut LayoutCtx, bc: &BoxConstraints, data: &T, env: &Env) -> Size {
        bc.debug_check("TabsBody");

        data.with_selected(|id, tab_data| {
            let body = self.children.get_mut(&id).unwrap();
            let size = body.layout(ctx, bc, tab_data, env);
            body.set_origin(ctx, tab_data, env, Point::ORIGIN);
            size
        })
        .unwrap_or_else(|| bc.min())
    }

    fn paint(&mut self, ctx: &mut PaintCtx, data: &T, env: &Env) {
        data.with_selected(|id, tab_data| {
            let bounds = ctx.size().to_rect();
            ctx.fill(bounds, &env.get(theme::SELECTED_TAB_BACKGROUND));

            self.children
                .get_mut(&id)
                .unwrap()
                .paint_raw(ctx, tab_data, env)
        });
    }
}

impl<T, G, W> TabsBody<T, G, W>
where
    T: TabsData,
{
    fn for_each<F>(&mut self, data: &T, mut f: F)
    where
        F: FnMut(TabId, &mut WidgetPod<T::Item, W>, &T::Item),
    {
        let mut children = self.children.iter_mut();
        data.for_each(|tab_id, tab_data| match children.next() {
            Some((&tab_id2, tab)) if tab_id == tab_id2 => f(tab_id, tab, tab_data),
            _ => tracing::error!("TabBody out of sync with data"),
        })
    }

    fn for_each_mut<F>(&mut self, data: &mut T, mut f: F)
    where
        F: FnMut(TabId, &mut WidgetPod<T::Item, W>, &mut T::Item),
    {
        let mut children = self.children.iter_mut();
        data.for_each_mut(|tab_id, tab_data| match children.next() {
            Some((&tab_id2, tab)) if tab_id == tab_id2 => f(tab_id, tab, tab_data),
            _ => tracing::error!("TabBody out of sync with data"),
        })
    }
}
