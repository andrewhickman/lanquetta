use std::borrow::Cow;

use druid::{Data, Env, Widget};
use druid::piet::TextStorage;

use crate::theme;

pub struct FormField<W, F> {
    pristine: bool,
    validate: F,
    child: W,
}

#[derive(Clone, Debug)]
pub struct ValidationState<T, O, E> {
    raw: T,
    result: Result<O, E>,
}

impl<W, F> FormField<W, F> {
    pub fn new(child: W, validate: F) -> Self {
        FormField {
            pristine: true,
            child: child,
            validate,
        }
    }

    pub fn set_validate(&mut self, validate: F) {
        self.validate = validate;
    }
}

impl<T, W, F, O, E> Widget<ValidationState<T, O, E>> for FormField<W, F>
where
    T: TextStorage,
    W: Widget<T>,
    F: Fn(&str) -> Result<O, E>,
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
        self.child.event(ctx, event, &mut data.raw, &env);
        self.pristine &= !ctx.is_focused();
        data.result = (self.validate)(data.raw.as_str());
    }

    fn lifecycle(
        &mut self,
        ctx: &mut druid::LifeCycleCtx,
        event: &druid::LifeCycle,
        data: &ValidationState<T, O, E>,
        env: &druid::Env,
    ) {
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

    fn paint(&mut self, ctx: &mut druid::PaintCtx, data: &ValidationState<T, O, E>, env: &druid::Env) {
        self.child
            .paint(ctx, &data.raw, &data.update_env(env, self.pristine))
    }
}

impl<T, O, E> ValidationState<T, O, E> {
    pub fn new(raw: T, result: Result<O, E>) -> Self {
        ValidationState { raw, result }
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
        // validator is assumed to be idempotent
        self.raw.same(&other.raw)
    }
}
