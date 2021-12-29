use std::fmt;
use std::sync::Arc;

use druid::{
    piet::TextStorage,
    widget::{prelude::*, Controller},
    LifeCycle, Point, Selector, WidgetExt, WidgetPod,
};
use druid::{Data, Env, Widget};

use crate::theme;

pub const FINISH_EDIT: Selector = Selector::new("app.finish-edit");

pub type ValidationFn<O, E> = Arc<dyn Fn(&str) -> Result<O, E> + Send + Sync>;

pub struct FormField<T> {
    child: WidgetPod<T, Box<dyn Widget<T>>>,
    env: Option<Env>,
    id: WidgetId,
}

#[derive(Clone)]
pub struct ValidationState<T, O, E> {
    raw: T,
    validate: ValidationFn<O, E>,
    result: Result<O, E>,
    pristine: bool,
}

struct FinishEditController {
    field_id: WidgetId,
}

impl<T: TextStorage> FormField<T> {
    pub fn new<W>(child: W) -> Self
    where
        T: Data,
        W: Widget<T> + 'static,
    {
        let id = WidgetId::next();
        FormField {
            child: WidgetPod::new(
                child
                    .controller(FinishEditController { field_id: id })
                    .boxed(),
            ),
            env: None,
            id,
        }
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
    T: Data + TextStorage,
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
            if command.is(FINISH_EDIT) {
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
        self.child.set_origin(ctx, &data.raw, env, Point::ORIGIN);
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

impl<T, O, E> ValidationState<T, O, E>
where
    T: TextStorage,
{
    pub fn new(raw: T, validate: ValidationFn<O, E>) -> Self {
        let result = validate(raw.as_str());
        ValidationState {
            raw,
            result,
            validate,
            pristine: true,
        }
    }

    pub fn dirty(raw: T, validate: ValidationFn<O, E>) -> Self {
        let result = validate(raw.as_str());
        ValidationState {
            raw,
            result,
            validate,
            pristine: false,
        }
    }

    pub fn result(&self) -> Result<&O, &E> {
        self.result.as_ref()
    }

    pub fn text(&self) -> &T {
        &self.raw
    }

    pub fn text_mut(&mut self) -> &mut T {
        &mut self.raw
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
    T: TextStorage + Data,
{
    fn with_text_mut<V>(&mut self, f: impl FnOnce(&mut T) -> V) -> V {
        let old = self.raw.clone();
        let value = f(&mut self.raw);
        if !self.raw.same(&old) {
            self.result = (self.validate)(self.raw.as_str());
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
            .finish()
    }
}

impl<T, W> Controller<T, W> for FinishEditController
where
    W: Widget<T>,
{
    fn lifecycle(
        &mut self,
        child: &mut W,
        ctx: &mut LifeCycleCtx,
        event: &LifeCycle,
        data: &T,
        env: &Env,
    ) {
        if let LifeCycle::FocusChanged(false) = event {
            ctx.submit_command(FINISH_EDIT.to(self.field_id));
        }

        child.lifecycle(ctx, event, data, env)
    }
}
