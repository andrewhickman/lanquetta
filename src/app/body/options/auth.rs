use std::{mem, sync::Arc};

use anyhow::Result;
use druid::{
    widget::{
        prelude::*, Button, Controller, CrossAxisAlignment, Either, Flex, Label, LineBreaking,
        Spinner, TextBox, ViewSwitcher,
    },
    ArcStr, Command, Handled, Lens, Selector, WidgetExt,
};
use once_cell::sync::Lazy;

use crate::{
    app::{body::layout_spinner, fmt_err},
    auth::AuthorizationHook,
    theme,
    widget::{
        update_queue::{self, UpdateQueue},
        Empty, FormField, Icon, ValidationFn, ValidationState,
    },
};

#[derive(Debug, Data, Clone, Lens)]
pub struct State {
    command: CommandValidationState,
    execute_state: ExecuteState,
}

#[derive(Debug, Data, Clone)]
pub enum ExecuteState {
    NotStarted,
    Active,
    Succeeded,
    Failed(ArcStr),
}

const START_TEST: Selector = Selector::new("app.body.options.auth.start-test");
const CANCEL_TEST: Selector = Selector::new("app.body.options.auth.cancel-test");

struct AuthOptionsController {
    updates: UpdateQueue<AuthOptionsController, State>,
}

type CommandValidationState = ValidationState<String, Option<Arc<AuthorizationHook>>, ArcStr>;

pub fn build() -> impl Widget<State> {
    let id = WidgetId::next();

    let command_textbox = FormField::text_box(theme::text_box_scope(
        TextBox::new()
            .with_placeholder(command_placeholder())
            .expand_width(),
    ))
    .lens(State::command);

    let error_label = theme::error_label_scope(
        Label::dynamic(|data: &State, _| {
            if let Err(err) = data.command.result() {
                err.to_string()
            } else if let ExecuteState::Failed(err) = &data.execute_state {
                err.to_string()
            } else {
                String::default()
            }
        })
        .with_line_break_mode(LineBreaking::WordWrap),
    );
    let error = Either::new(
        |data: &State, _| {
            !data.command.is_pristine_or_valid()
                || matches!(data.execute_state, ExecuteState::Failed(_))
        },
        error_label,
        Empty,
    )
    .expand_width();

    let command_form_field = Flex::column().with_child(command_textbox).with_child(error);

    let spinner = ViewSwitcher::new(
        |execute_state: &ExecuteState, _| mem::discriminant(execute_state),
        |_, execute_state, _| match execute_state {
            ExecuteState::NotStarted => Empty.boxed(),
            ExecuteState::Active => layout_spinner(Spinner::new(), 2.0),
            ExecuteState::Succeeded => {
                layout_spinner(Icon::check().with_color(theme::color::BOLD_ACCENT), 0.0)
            }
            ExecuteState::Failed(_) => {
                layout_spinner(Icon::close().with_color(theme::color::ERROR), 0.0)
            }
        },
    );

    let test_button = theme::button_scope(Button::new("Test").on_click(
        move |ctx: &mut EventCtx, _: &mut State, _: &Env| {
            ctx.submit_command(START_TEST.to(id));
        },
    ))
    .disabled_if(|data: &State, _| {
        matches!(data.command.result(), Ok(None) | Err(_))
            || matches!(data.execute_state, ExecuteState::Active)
    });

    Flex::row()
        .cross_axis_alignment(CrossAxisAlignment::Start)
        .with_flex_child(command_form_field, 1.0)
        .with_spacer(theme::BODY_SPACER)
        .with_child(spinner.lens(State::execute_state))
        .with_child(test_button.fix_width(100.0))
        .controller(AuthOptionsController::new())
        .with_id(id)
}

impl State {
    pub fn new(auth_hook: &Option<Arc<AuthorizationHook>>) -> State {
        let shell = match auth_hook {
            Some(hook) => hook.shell(),
            None => "",
        };

        State {
            command: ValidationState::new(shell.to_owned(), VALIDATE_COMMAND.clone()),
            execute_state: ExecuteState::NotStarted,
        }
    }

    pub fn hook(&self) -> Option<Arc<AuthorizationHook>> {
        self.command.result().ok().and_then(|h| h.clone())
    }
}

impl AuthOptionsController {
    fn new() -> Self {
        AuthOptionsController {
            updates: UpdateQueue::new(),
        }
    }

    fn command(
        &mut self,
        ctx: &mut EventCtx<'_, '_>,
        command: &Command,
        data: &mut State,
    ) -> Handled {
        if command.is(START_TEST) {
            let Ok(Some(hook)) = data.command.result() else {
                tracing::warn!("start-test called without authorization hook");
                return Handled::Yes;
            };

            let writer = self.updates.writer(ctx);
            let hook = hook.clone();

            data.execute_state = ExecuteState::Active;
            tokio::spawn(async move {
                let result = hook.get_headers_force().await.map(drop);
                writer.write(|_, _, data| match result {
                    Ok(_) => data.execute_state = ExecuteState::Succeeded,
                    Err(err) => data.execute_state = ExecuteState::Failed(fmt_err(&err)),
                });
            });

            Handled::Yes
        } else if command.is(CANCEL_TEST) {
            data.execute_state = ExecuteState::NotStarted;
            self.updates.disconnect();
            Handled::Yes
        } else if command.is(update_queue::UPDATE) {
            while let Some(update) = self.updates.pop() {
                (update)(self, ctx, data)
            }
            Handled::Yes
        } else {
            Handled::No
        }
    }
}

impl<W> Controller<State, W> for AuthOptionsController
where
    W: Widget<State>,
{
    fn event(
        &mut self,
        child: &mut W,
        ctx: &mut EventCtx,
        event: &Event,
        data: &mut State,
        env: &Env,
    ) {
        match event {
            Event::Command(command) if self.command(ctx, command, data) == Handled::Yes => (),
            _ => child.event(ctx, event, data, env),
        }
    }

    fn update(
        &mut self,
        child: &mut W,
        ctx: &mut UpdateCtx,
        old_data: &State,
        data: &State,
        env: &Env,
    ) {
        if old_data.command.text() != data.command.text() {
            ctx.submit_command(CANCEL_TEST.to(ctx.widget_id()));
        }

        child.update(ctx, old_data, data, env)
    }
}

static VALIDATE_COMMAND: Lazy<ValidationFn<String, Option<Arc<AuthorizationHook>>, ArcStr>> =
    Lazy::new(|| Arc::new(validate_hook));

fn validate_hook(s: &String) -> Result<Option<Arc<AuthorizationHook>>, ArcStr> {
    if s.is_empty() {
        return Ok(None);
    }

    let hook = AuthorizationHook::new(s.clone()).map_err(|err| fmt_err(&err))?;

    Ok(Some(Arc::new(hook)))
}

fn command_placeholder() -> String {
    if cfg!(windows) {
        "powershell generate_token.ps1".to_owned()
    } else {
        "bash generate_token.sh".to_owned()
    }
}
