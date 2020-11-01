use std::fmt;
use std::sync::Arc;

use druid::{piet::TextStorage, LifeCycle, WidgetPod};
use druid::{Data, Env, Widget};

use crate::theme;

pub struct FormField<T, W> {
    pristine: bool,
    child: WidgetPod<T, W>,
    env: Option<Env>,
}

#[derive(Clone)]
pub struct ValidationState<T, O, E> {
    raw: T,
    validate: Arc<dyn Fn(&str) -> Result<O, E>>,
    result: Result<O, E>,
}

impl<T, W> FormField<T, W> {
    pub fn new(child: W) -> Self
    where
        W: Widget<T>,
    {
        FormField {
            pristine: true,
            child: WidgetPod::new(child),
            env: None,
        }
    }

    fn is_valid_or_pristine<O, E>(&self, data: &ValidationState<T, O, E>) -> bool {
        data.is_valid() || self.pristine
    }

    fn update_env<O, E>(&mut self, data: &ValidationState<T, O, E>, env: &Env) -> bool {
        if self.is_valid_or_pristine(data) != self.env.is_none() {
            self.env = if self.is_valid_or_pristine(data) {
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

impl<T, W, O, E> Widget<ValidationState<T, O, E>> for FormField<T, W>
where
    T: TextStorage,
    W: Widget<T>,
    T: Data,
    ValidationState<T, O, E>: Clone + 'static,
{
    fn event(
        &mut self,
        ctx: &mut druid::EventCtx,
        event: &druid::Event,
        data: &mut ValidationState<T, O, E>,
        env: &druid::Env,
    ) {
        data.with_text_mut_if_changed(|text| {
            self.child
                .event(ctx, event, text, self.env.as_ref().unwrap_or(env))
        });
    }

    fn lifecycle(
        &mut self,
        ctx: &mut druid::LifeCycleCtx,
        event: &druid::LifeCycle,
        data: &ValidationState<T, O, E>,
        env: &druid::Env,
    ) {
        match event {
            LifeCycle::WidgetAdded => ctx.children_changed(),
            LifeCycle::FocusChanged(false) => {
                self.pristine = false;
                if self.update_env(data, &env) {
                    ctx.request_paint();
                }
            }
            _ => (),
        }

        self.child
            .lifecycle(ctx, event, &data.raw, self.env.as_ref().unwrap_or(env));
    }

    fn update(
        &mut self,
        ctx: &mut druid::UpdateCtx,
        _: &ValidationState<T, O, E>,
        data: &ValidationState<T, O, E>,
        env: &druid::Env,
    ) {
        if ctx.env_changed() {
            self.env = None;
        }
        self.update_env(data, &env);

        self.child
            .update(ctx, &data.raw, self.env.as_ref().unwrap_or(env));

        if ctx.env_changed() {
            self.env = None;
        }
        if self.update_env(data, &env) {
            ctx.request_paint();
        }
    }

    fn layout(
        &mut self,
        ctx: &mut druid::LayoutCtx,
        bc: &druid::BoxConstraints,
        data: &ValidationState<T, O, E>,
        env: &druid::Env,
    ) -> druid::Size {
        let env = self.env.as_ref().unwrap_or(env);
        let size = self.child.layout(ctx, bc, &data.raw, &env);
        self.child.set_layout_rect(ctx, &data.raw, &env, size.to_rect());
        size
    }

    fn paint(
        &mut self,
        ctx: &mut druid::PaintCtx,
        data: &ValidationState<T, O, E>,
        env: &druid::Env,
    ) {
        self.child
            .paint(ctx, &data.raw, self.env.as_ref().unwrap_or(env))
    }
}

impl<T, O, E> ValidationState<T, O, E>
where
    T: TextStorage,
{
    pub fn new(raw: T, validate: Arc<dyn Fn(&str) -> Result<O, E>>) -> Self {
        let result = validate(raw.as_str());
        ValidationState {
            raw,
            result,
            validate,
        }
    }

    pub fn result(&self) -> Result<&O, &E> {
        self.result.as_ref()
    }

    pub fn with_text_mut<V>(&mut self, f: impl FnOnce(&mut T) -> V) -> V {
        let value = f(&mut self.raw);
        self.result = (self.validate)(self.raw.as_str());
        value
    }

    pub fn text(&self) -> &T {
        &self.raw
    }
}

impl<T, O, E> ValidationState<T, O, E> {
    pub fn is_valid(&self) -> bool {
        self.result.is_ok()
    }
}

impl<T, O, E> ValidationState<T, O, E>
where
    T: TextStorage + Data,
{
    fn with_text_mut_if_changed<V>(&mut self, f: impl FnOnce(&mut T) -> V) -> V {
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
