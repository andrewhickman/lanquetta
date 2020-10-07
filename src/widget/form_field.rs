use std::borrow::Cow;

use druid::{Data, Env, Widget};

use crate::theme;

pub struct FormField<W, F> {
    pristine: bool,
    validate: F,
    child: W,
}

#[derive(Clone, Debug)]
pub struct ValidationState<O, E> {
    raw: String,
    result: Result<O, E>,
}

struct Validator<F> {
    validate: F,
}

impl<W, F, O, E> FormField<W, F>
where
    W: Widget<String> + 'static,
    F: Fn(&str) -> Result<O, E>,
    ValidationState<O, E>: Data,
{
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

impl<W, F, O, E> Widget<ValidationState<O, E>> for FormField<W, F>
where
    W: Widget<String>,
    F: Fn(&str) -> Result<O, E>,
    ValidationState<O, E>: Data,
{
    fn event(
        &mut self,
        ctx: &mut druid::EventCtx,
        event: &druid::Event,
        data: &mut ValidationState<O, E>,
        env: &druid::Env,
    ) {
        let env = data.update_env(env, self.pristine);
        self.child.event(ctx, event, &mut data.raw, &env);
        self.pristine &= !ctx.is_focused();
        data.result = (self.validate)(&data.raw);
    }

    fn lifecycle(
        &mut self,
        ctx: &mut druid::LifeCycleCtx,
        event: &druid::LifeCycle,
        data: &ValidationState<O, E>,
        env: &druid::Env,
    ) {
        self.child
            .lifecycle(ctx, event, &data.raw, &data.update_env(env, self.pristine));
    }

    fn update(
        &mut self,
        ctx: &mut druid::UpdateCtx,
        old_data: &ValidationState<O, E>,
        data: &ValidationState<O, E>,
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
        data: &ValidationState<O, E>,
        env: &druid::Env,
    ) -> druid::Size {
        self.child
            .layout(ctx, bc, &data.raw, &data.update_env(env, self.pristine))
    }

    fn paint(&mut self, ctx: &mut druid::PaintCtx, data: &ValidationState<O, E>, env: &druid::Env) {
        self.child
            .paint(ctx, &data.raw, &data.update_env(env, self.pristine))
    }
}

impl<O, E> ValidationState<O, E> {
    pub fn new(raw: String, result: Result<O, E>) -> Self {
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

impl<O, E> Data for ValidationState<O, E>
where
    Self: Clone + 'static,
{
    fn same(&self, other: &Self) -> bool {
        // validator is assumed to be idempotent
        self.raw.same(&other.raw)
    }
}
