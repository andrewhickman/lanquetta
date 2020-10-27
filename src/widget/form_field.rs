use std::fmt;
use std::{borrow::Cow, sync::Arc};

use druid::{piet::TextStorage, LifeCycle};
use druid::{Data, Env, Widget};

use crate::theme;

pub struct FormField<W> {
    pristine: bool,
    child: W,
}

#[derive(Clone)]
pub struct ValidationState<T, O, E> {
    raw: T,
    validate: Arc<dyn Fn(&str) -> Result<O, E>>,
    result: Result<O, E>,
}

impl<W> FormField<W> {
    pub fn new(child: W) -> Self {
        FormField {
            pristine: true,
            child,
        }
    }
}

impl<T, W, O, E> Widget<ValidationState<T, O, E>> for FormField<W>
where
    T: TextStorage,
    W: Widget<T>,
    ValidationState<T, O, E>: Data,
{
    fn event(
        &mut self,
        ctx: &mut druid::EventCtx,
        event: &druid::Event,
        data: &mut ValidationState<T, O, E>,
        env: &druid::Env,
    ) {
        let env = data.update_env(env, self.pristine);
        data.with_text_mut(|text| self.child.event(ctx, event, text, &env));
    }

    fn lifecycle(
        &mut self,
        ctx: &mut druid::LifeCycleCtx,
        event: &druid::LifeCycle,
        data: &ValidationState<T, O, E>,
        env: &druid::Env,
    ) {
        if let LifeCycle::FocusChanged(false) = event {
            self.pristine = false;
        }

        self.child
            .lifecycle(ctx, event, &data.raw, &data.update_env(env, self.pristine));
    }

    fn update(
        &mut self,
        ctx: &mut druid::UpdateCtx,
        old_data: &ValidationState<T, O, E>,
        data: &ValidationState<T, O, E>,
        env: &druid::Env,
    ) {
        self.child.update(
            ctx,
            &old_data.raw,
            &data.raw,
            &data.update_env(env, self.pristine),
        );
    }

    fn layout(
        &mut self,
        ctx: &mut druid::LayoutCtx,
        bc: &druid::BoxConstraints,
        data: &ValidationState<T, O, E>,
        env: &druid::Env,
    ) -> druid::Size {
        self.child
            .layout(ctx, bc, &data.raw, &data.update_env(env, self.pristine))
    }

    fn paint(
        &mut self,
        ctx: &mut druid::PaintCtx,
        data: &ValidationState<T, O, E>,
        env: &druid::Env,
    ) {
        self.child
            .paint(ctx, &data.raw, &data.update_env(env, self.pristine))
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

    fn is_valid(&self) -> bool {
        self.result.is_ok()
    }

    fn update_env<'a>(&self, env: &'a Env, pristine: bool) -> Cow<'a, Env> {
        if pristine || self.is_valid() {
            Cow::Borrowed(env)
        } else {
            let mut env = env.clone();
            theme::set_error(&mut env);
            Cow::Owned(env)
        }
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
