use std::fmt;
use std::sync::Arc;

use druid::{
    widget::{prelude::*, Controller},
    LifeCycle, Point, Selector, WidgetExt, WidgetPod, text::TextComponent,
};
use druid::{Data, Env, Widget};

use crate::theme;

pub const START_EDIT: Selector = Selector::new("app.start-edit");
pub const FINISH_EDIT: Selector = Selector::new("app.finish-edit");

pub type ValidationFn<T, O, E> = Arc<dyn Fn(&T) -> Result<O, E> + Send + Sync>;

pub struct FormField<T> {
    child: WidgetPod<T, Box<dyn Widget<T>>>,
    env: Option<Env>,
    id: WidgetId,
}

#[derive(Clone)]
pub struct ValidationState<T, O, E> {
    raw: T,
    validate: ValidationFn<T, O, E>,
    result: Result<O, E>,
    pristine: bool,
    editing: bool,
}

pub struct FinishEditController {
    field_id: WidgetId,
}

impl<T> FormField<T> {
    pub fn new<W>(id: WidgetId, child: W) -> Self
    where
        T: Data,
        W: Widget<T> + 'static,
    {
        FormField {
            child: WidgetPod::new(child).boxed(),
            env: None,
            id,
        }
    }

    pub fn text_box<W>(child: W) -> Self
    where
        T: Data,
        W: Widget<T> + 'static,
    {
        let id = WidgetId::next();
        FormField::new(id, child.controller(FinishEditController::new(id)))
    }

    fn update_env<O, E>(&mut self, data: &ValidationState<T, O, E>, env: &Env) -> bool {
        if data.is_pristine_or_valid() != self.env.is_none() {
            self.env = if data.is_pristine_or_valid() {
                None
            } else {
                Some(env.clone().adding(theme::INVALID, true))
            };
            true
        } else {
            false
        }
    }
}

impl<T, O, E> Widget<ValidationState<T, O, E>> for FormField<T>
where
    T: Data,
    ValidationState<T, O, E>: Clone + 'static,
{
    fn event(
        &mut self,
        ctx: &mut EventCtx,
        event: &Event,
        data: &mut ValidationState<T, O, E>,
        env: &Env,
    ) {
        if let Event::Command(command) = event {
            if command.is(START_EDIT) {
                data.editing = true;
            } else if data.editing && command.is(FINISH_EDIT) {
                data.editing = false;
                data.set_dirty();
            }
        }

        data.with_text_mut(|text| {
            self.child
                .event(ctx, event, text, self.env.as_ref().unwrap_or(env))
        });
    }

    fn lifecycle(
        &mut self,
        ctx: &mut LifeCycleCtx,
        event: &LifeCycle,
        data: &ValidationState<T, O, E>,
        env: &Env,
    ) {
        if let LifeCycle::WidgetAdded = event {
            self.update_env(data, env);
            ctx.children_changed();
        }

        self.child
            .lifecycle(ctx, event, &data.raw, self.env.as_ref().unwrap_or(env));
    }

    fn update(
        &mut self,
        ctx: &mut UpdateCtx,
        _: &ValidationState<T, O, E>,
        data: &ValidationState<T, O, E>,
        env: &Env,
    ) {
        if ctx.env_changed() {
            self.env = None;
        }
        self.update_env(data, env);

        self.child
            .update(ctx, &data.raw, self.env.as_ref().unwrap_or(env));

        if ctx.env_changed() {
            self.env = None;
        }
        if self.update_env(data, env) {
            ctx.request_paint();
        }
    }

    fn layout(
        &mut self,
        ctx: &mut LayoutCtx,
        bc: &BoxConstraints,
        data: &ValidationState<T, O, E>,
        env: &Env,
    ) -> druid::Size {
        bc.debug_check("FormField");

        let env = self.env.as_ref().unwrap_or(env);
        let size = self.child.layout(ctx, bc, &data.raw, env);
        self.child.set_origin(ctx, Point::ORIGIN);
        size
    }

    fn paint(&mut self, ctx: &mut PaintCtx, data: &ValidationState<T, O, E>, env: &Env) {
        self.child
            .paint(ctx, &data.raw, self.env.as_ref().unwrap_or(env))
    }

    fn id(&self) -> Option<WidgetId> {
        Some(self.id)
    }
}

impl<T, O, E> ValidationState<T, O, E> {
    pub fn new(raw: T, validate: ValidationFn<T, O, E>) -> Self {
        let result = validate(&raw);
        ValidationState {
            raw,
            result,
            validate,
            pristine: true,
            editing: false,
        }
    }

    pub fn dirty(raw: T, validate: ValidationFn<T, O, E>) -> Self {
        let result = validate(&raw);
        ValidationState {
            raw,
            result,
            validate,
            pristine: false,
            editing: false,
        }
    }

    pub fn result(&self) -> Result<&O, &E> {
        self.result.as_ref()
    }

    pub fn text(&self) -> &T {
        &self.raw
    }
}

impl<T, O, E> ValidationState<T, O, E> {
    pub fn is_valid(&self) -> bool {
        self.result.is_ok()
    }

    pub fn is_pristine_or_valid(&self) -> bool {
        self.pristine || self.is_valid()
    }

    pub fn set_dirty(&mut self) {
        self.pristine = false;
    }
}

impl<T, O, E> ValidationState<T, O, E>
where
    E: Clone,
{
    pub fn error(&self) -> Option<E> {
        self.result().err().cloned()
    }

    pub fn display_error(&self) -> Option<E> {
        if self.pristine {
            None
        } else {
            self.error()
        }
    }
}

impl<T, O, E> ValidationState<T, O, E>
where
    T: Data,
{
    pub fn with_text_mut<V>(&mut self, f: impl FnOnce(&mut T) -> V) -> V {
        let old = self.raw.clone();
        let value = f(&mut self.raw);
        if !self.raw.same(&old) {
            self.result = (self.validate)(&self.raw);
        }
        value
    }
}

impl<T, O, E> Data for ValidationState<T, O, E>
where
    T: Data,
    Self: Clone + 'static,
{
    fn same(&self, other: &Self) -> bool {
        self.raw.same(&other.raw)
            && self.validate.same(&other.validate)
            && self.pristine.same(&other.pristine)
            && self.editing.same(&other.editing)
    }
}

impl<T, O, E> fmt::Debug for ValidationState<T, O, E>
where
    T: fmt::Debug,
    Result<O, E>: fmt::Debug,
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("ValidationState")
            .field("raw", &self.raw)
            .field("result", &self.result)
            .field("pristine", &self.pristine)
            .field("editing", &self.editing)
            .finish()
    }
}

impl FinishEditController {
    pub fn new(parent: WidgetId) -> Self {
        FinishEditController { field_id: parent }
    }
}

impl<T, W> Controller<T, W> for FinishEditController
where
    W: Widget<T>,
{
    fn event(&mut self, child: &mut W, ctx: &mut EventCtx, event: &Event, data: &mut T, env: &Env) {
        if let Event::Notification(note) = event {
            if note.is(TextComponent::RETURN) {
                ctx.submit_command(FINISH_EDIT.to(self.field_id));
                ctx.set_handled();
            } else if note.is(TextComponent::CANCEL) {
                ctx.submit_command(FINISH_EDIT.to(self.field_id));
                ctx.resign_focus();
                ctx.set_handled();
            }
        }

        child.event(ctx, event, data, env)
    }

    fn lifecycle(
        &mut self,
        child: &mut W,
        ctx: &mut LifeCycleCtx,
        event: &LifeCycle,
        data: &T,
        env: &Env,
    ) {
        if let LifeCycle::FocusChanged(focused) = event {
            if *focused {
                ctx.submit_command(START_EDIT.to(self.field_id));
            } else {
                ctx.submit_command(FINISH_EDIT.to(self.field_id));
            }
        }

        child.lifecycle(ctx, event, data, env)
    }
}
