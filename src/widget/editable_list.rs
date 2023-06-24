use std::sync::Arc;

use druid::{
    widget::{prelude::*, Flex, Label, LineBreaking},
    widget::{List, ListIter},
    Lens, Point, WidgetExt, WidgetPod,
};

use crate::{
    theme::{self, GRID_NARROW_SPACER},
    widget::Icon,
};

pub struct EditableList<T> {
    add_button: WidgetPod<Arc<Vec<T>>, Box<dyn Widget<Arc<Vec<T>>>>>,
    entries: WidgetPod<Arc<Vec<T>>, Box<dyn Widget<Arc<Vec<T>>>>>,
}

impl<T> EditableList<T>
where
    T: Data,
{
    pub fn new<W>(
        add_label: impl Into<String>,
        on_add: impl Fn(&mut EventCtx, &mut Arc<Vec<T>>, &Env) + 'static,
        make_widget: impl Fn() -> W + 'static,
    ) -> Self
    where
        T: Data,
        W: Widget<T> + 'static,
    {
        EditableList {
            add_button: WidgetPod::new(
                Flex::row()
                    .with_child(Icon::add().padding(3.0))
                    .with_child(
                        Label::new(add_label.into())
                            .with_font(theme::font::HEADER_TWO)
                            .with_line_break_mode(LineBreaking::Clip),
                    )
                    .must_fill_main_axis(true)
                    .on_click(on_add)
                    .background(theme::hot_or_active_painter(
                        druid::theme::BUTTON_BORDER_RADIUS,
                    ))
                    .boxed(),
            ),
            entries: WidgetPod::new(
                List::new(move || {
                    Flex::row()
                        .with_flex_child(make_widget().lens(Entry::item), 1.0)
                        .with_spacer(GRID_NARROW_SPACER)
                        .with_child(Icon::close().button(move |_, data: &mut Entry<T>, _| {
                            data.deleted = true;
                        }))
                })
                .with_spacing(GRID_NARROW_SPACER)
                .scroll()
                .vertical()
                .boxed(),
            ),
        }
    }
}

#[derive(Clone, Data, Lens)]
struct Entry<T> {
    item: T,
    #[lens(ignore)]
    deleted: bool,
}

impl<T> ListIter<Entry<T>> for Arc<Vec<T>>
where
    T: Data,
{
    fn for_each(&self, mut cb: impl FnMut(&Entry<T>, usize)) {
        for (index, item) in self.iter().enumerate() {
            cb(
                &Entry {
                    item: item.clone(),
                    deleted: false,
                },
                index,
            )
        }
    }

    fn for_each_mut(&mut self, mut cb: impl FnMut(&mut Entry<T>, usize)) {
        let mut new_list: Option<Arc<Vec<T>>> = None;

        for (index, item) in self.iter().enumerate() {
            let mut entry = Entry {
                item: item.clone(),
                deleted: false,
            };

            cb(&mut entry, index);

            if entry.deleted || !item.same(&entry.item) {
                let new_list = Arc::make_mut(new_list.get_or_insert_with(|| self.clone()));
                let new_index = index - (self.len() - new_list.len());

                if entry.deleted {
                    new_list.remove(new_index);
                } else {
                    new_list[new_index] = entry.item;
                }
            }
        }

        if let Some(new_list) = new_list {
            *self = new_list;
        }
    }

    fn data_len(&self) -> usize {
        self.len()
    }
}

impl<T> Widget<Arc<Vec<T>>> for EditableList<T>
where
    T: Data,
{
    fn event(&mut self, ctx: &mut EventCtx, event: &Event, data: &mut Arc<Vec<T>>, env: &Env) {
        self.entries.event(ctx, event, data, env);
        self.add_button.event(ctx, event, data, env);
    }

    fn lifecycle(
        &mut self,
        ctx: &mut LifeCycleCtx,
        event: &LifeCycle,
        data: &Arc<Vec<T>>,
        env: &Env,
    ) {
        self.entries.lifecycle(ctx, event, data, env);
        self.add_button.lifecycle(ctx, event, data, env);
    }

    fn update(
        &mut self,
        ctx: &mut UpdateCtx,
        old_data: &Arc<Vec<T>>,
        data: &Arc<Vec<T>>,
        env: &Env,
    ) {
        if !old_data.same(data) {
            self.entries.update(ctx, data, env);
            self.add_button.update(ctx, data, env);
        }
    }

    fn layout(
        &mut self,
        ctx: &mut LayoutCtx,
        bc: &BoxConstraints,
        data: &Arc<Vec<T>>,
        env: &Env,
    ) -> Size {
        let width = bc.max().width;
        let max_height = (bc.max().height - GRID_NARROW_SPACER).max(bc.min().height);
        let tight_bc = BoxConstraints::new(
            Size::new(width, bc.min().height),
            Size::new(width, max_height),
        );

        let add_button_size = self.add_button.layout(ctx, &tight_bc, data, env);

        let metadata_bc = tight_bc
            .shrink_max_height_to(bc.max().height - add_button_size.height - GRID_NARROW_SPACER);
        let metadata_size = self.entries.layout(ctx, &metadata_bc, data, env);

        self.entries.set_origin(ctx, Point::ORIGIN);
        self.add_button.set_origin(
            ctx,
            Point::new(0.0, metadata_size.height + GRID_NARROW_SPACER),
        );

        bc.constrain(Size::new(
            width,
            add_button_size.height + GRID_NARROW_SPACER + metadata_size.height,
        ))
    }

    fn paint(&mut self, ctx: &mut PaintCtx, data: &Arc<Vec<T>>, env: &Env) {
        self.entries.paint(ctx, data, env);
        self.add_button.paint(ctx, data, env);
    }
}
