use std::fmt;
use std::sync::Arc;

use druid::{
    text::TextComponent,
    widget::{prelude::*, Controller},
    ArcStr, Key, LifeCycle, Point, Selector, WidgetExt, WidgetPod,
};
use druid::{Data, Env, Widget};

use crate::theme;

pub const START_EDIT: Selector = Selector::new("app.form-field.start-edit");
pub const FINISH_EDIT: Selector = Selector::new("app.form-field.finish-edit");
pub const REFRESH: Selector = Selector::new("app.form-field.refresh");
pub const ERROR_MESSAGE: Key<ArcStr> = Key::new("app.form-field.error-message");

pub type ValidationFn<T, O> = Arc<dyn Fn(&T) -> Result<O, ArcStr> + Send + Sync>;

pub struct FormField<T> {
    child: WidgetPod<T, Box<dyn Widget<T>>>,
    env: Option<Env>,
    id: WidgetId,
}

#[derive(Clone)]
pub struct ValidationState<T, O> {
    raw: T,
    validate: ValidationFn<T, O>,
    result: Result<O, ArcStr>,
    pristine: bool,
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

    fn update_env<O>(&mut self, data: &ValidationState<T, O>, env: &Env) -> bool {
        if data.is_pristine_or_valid() != self.env.is_none() {
            self.env = if data.pristine {
                None
            } else if let Err(err) = data.result() {
                Some(
                    env.clone()
                        .adding(theme::INVALID, true)
                        .adding(ERROR_MESSAGE, err.clone()),
                )
            } else {
                None
            };
            true
        } else {
            false
        }
    }
}

impl<T, O> Widget<ValidationState<T, O>> for FormField<T>
where
    T: Data,
    ValidationState<T, O>: Clone + 'static,
{
    fn event(
        &mut self,
        ctx: &mut EventCtx,
        event: &Event,
        data: &mut ValidationState<T, O>,
        env: &Env,
    ) {
        if let Event::Command(command) = event {
            if command.is(FINISH_EDIT) {
                data.set_dirty();
            } else if command.is(REFRESH) {
                data.refresh();
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
        data: &ValidationState<T, O>,
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
        _: &ValidationState<T, O>,
        data: &ValidationState<T, O>,
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
        data: &ValidationState<T, O>,
        env: &Env,
    ) -> druid::Size {
        bc.debug_check("FormField");

        let env = self.env.as_ref().unwrap_or(env);
        let size = self.child.layout(ctx, bc, &data.raw, env);
        self.child.set_origin(ctx, Point::ORIGIN);
        size
    }

    fn paint(&mut self, ctx: &mut PaintCtx, data: &ValidationState<T, O>, env: &Env) {
        self.child
            .paint(ctx, &data.raw, self.env.as_ref().unwrap_or(env))
    }

    fn id(&self) -> Option<WidgetId> {
        Some(self.id)
    }
}

impl<T, O> ValidationState<T, O> {
    pub fn new(raw: T, validate: ValidationFn<T, O>) -> Self {
        let result = validate(&raw);
        ValidationState {
            raw,
            result,
            validate,
            pristine: true,
        }
    }

    pub fn dirty(raw: T, validate: ValidationFn<T, O>) -> Self {
        let result = validate(&raw);
        ValidationState {
            raw,
            result,
            validate,
            pristine: false,
        }
    }

    pub fn result(&self) -> Result<&O, &ArcStr> {
        self.result.as_ref()
    }

    pub fn text(&self) -> &T {
        &self.raw
    }
}

impl<T, O> ValidationState<T, O> {
    pub fn is_valid(&self) -> bool {
        self.result.is_ok()
    }

    pub fn is_pristine_or_valid(&self) -> bool {
        self.pristine || self.is_valid()
    }

    pub fn set_dirty(&mut self) {
        self.pristine = false;
    }

    pub fn error(&self) -> Option<ArcStr> {
        self.result().err().cloned()
    }

    pub fn display_error(&self) -> Option<ArcStr> {
        if self.pristine {
            None
        } else {
            self.error()
        }
    }
}

impl<T, O> ValidationState<T, O>
where
    T: Data,
{
    pub fn with_text_mut<V>(&mut self, f: impl FnOnce(&mut T) -> V) -> V {
        let old = self.raw.clone();
        let value = f(&mut self.raw);
        if !self.raw.same(&old) {
            self.refresh();
        }
        value
    }

    pub fn refresh(&mut self) {
        self.result = (self.validate)(&self.raw);
    }
}

impl<T, O> Data for ValidationState<T, O>
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

impl<T, O> fmt::Debug for ValidationState<T, O>
where
    T: fmt::Debug,
    O: fmt::Debug,
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("ValidationState")
            .field("raw", &self.raw)
            .field("result", &self.result)
            .field("pristine", &self.pristine)
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
