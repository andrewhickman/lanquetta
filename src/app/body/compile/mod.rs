use std::{path::PathBuf, sync::Arc};

use druid::{
    widget::{
        prelude::*, Button, Controller, CrossAxisAlignment, Either, FillStrat, Flex, Label,
        LineBreaking, List, Scroll, TextBox,
    },
    ArcStr, Data, Lens, Selector, UnitPoint, WidgetExt,
};
use once_cell::sync::Lazy;

use crate::{
    theme::{self, BODY_PADDING, BODY_SPACER, GRID_NARROW_SPACER},
    widget::{Empty, FinishEditController, FormField, Icon, ValidationFn, ValidationState},
};

#[derive(Debug, Clone, Data, Lens)]
pub struct CompileTabState {
    error: Option<ArcStr>,
    #[lens(name = "includes_lens")]
    includes: Arc<Vec<PathValidationState>>,
    #[lens(name = "files_lens")]
    files: Arc<Vec<PathValidationState>>,
}

type PathValidationState = ValidationState<PathEntry, PathBuf, ArcStr>;

const DELETE_PATH: Selector = Selector::new("app.body.compile.delete-path");

#[derive(Default, Debug, Clone, Data, Lens)]
struct PathEntry {
    path: Arc<String>,
    deleted: bool,
}

pub fn build_body() -> impl Widget<CompileTabState> {
    let error = Either::new(
        |data: &Option<ArcStr>, _| data.is_some(),
        theme::error_label_scope(
            Label::dynamic(|data: &Option<ArcStr>, _| match data {
                Some(data) => data.to_string(),
                None => String::default(),
            })
            .with_line_break_mode(LineBreaking::WordWrap)
            .padding((0.0, BODY_PADDING, 0.0, 0.0)),
        ),
        Empty,
    )
    .lens(CompileTabState::error);

    let compile_button = theme::button_scope(Button::new("Compile").on_click(
        move |_: &mut EventCtx, data: &mut CompileTabState, _: &Env| {
            debug_assert!(data.is_valid());
            dbg!(data.includes().collect::<Vec<_>>());
            dbg!(data.files().collect::<Vec<_>>());
            data.error = Some("nope".into());
        },
    ))
    .disabled_if(|data: &CompileTabState, _| !data.is_valid());

    let compile_row = Flex::row()
        .with_flex_child(error, 1.0)
        .with_spacer(BODY_SPACER)
        .with_child(compile_button);

    Scroll::new(
        Flex::column()
            .with_child(
                Label::new("Include paths")
                    .with_font(theme::font::HEADER_TWO)
                    .align_left(),
            )
            .with_spacer(theme::BODY_SPACER)
            .with_child(
                build_path_list(VALIDATE_INCLUDE.clone()).lens(CompileTabState::includes_lens),
            )
            .with_spacer(theme::BODY_SPACER)
            .with_child(
                Label::new("Include paths")
                    .with_font(theme::font::HEADER_TWO)
                    .align_left(),
            )
            .with_spacer(theme::BODY_SPACER)
            .with_child(build_path_list(VALIDATE_FILE.clone()).lens(CompileTabState::files_lens))
            .with_child(compile_row)
            .padding(BODY_PADDING),
    )
    .vertical()
    .expand_height()
}

fn build_path_list(
    validator: ValidationFn<PathEntry, PathBuf, ArcStr>,
) -> impl Widget<Arc<Vec<PathValidationState>>> {
    let parent = WidgetId::next();

    Flex::column()
        .with_child(List::new(move || build_path_row(parent)))
        .with_child(build_add_path_button(validator))
        .controller(DeletePathController)
        .with_id(parent)
}

fn build_path_row(parent: WidgetId) -> impl Widget<PathValidationState> {
    let close = Icon::close()
        .with_fill(FillStrat::ScaleDown)
        .background(theme::hot_or_active_painter(
            druid::theme::BUTTON_BORDER_RADIUS,
        ))
        .on_click(
            move |ctx: &mut EventCtx, data: &mut PathValidationState, _| {
                data.with_text_mut(|state| state.deleted = true);
                ctx.submit_command(DELETE_PATH.to(parent));
            },
        );

    let form_id = WidgetId::next();
    let form_field = FormField::new(
        form_id,
        theme::text_box_scope(TextBox::<Arc<String>>::default().expand_width())
            .controller(FinishEditController::new(form_id))
            .lens(PathEntry::path),
    );

    let error = Either::new(
        |data: &PathValidationState, _: &Env| data.is_pristine_or_valid(),
        Empty,
        theme::error_label_scope(
            Label::dynamic(|data: &PathValidationState, _| {
                if let Err(err) = data.result() {
                    err.to_string()
                } else {
                    String::default()
                }
            })
            .align_vertical(UnitPoint::CENTER),
        )
        .padding((GRID_NARROW_SPACER, 0.0, 0.0, 0.0)),
    );

    Flex::row()
        .cross_axis_alignment(CrossAxisAlignment::Fill)
        .with_flex_child(form_field, 1.0)
        .with_spacer(GRID_NARROW_SPACER)
        .with_child(close)
        .with_child(error)
        .padding((0.0, 0.0, 0.0, GRID_NARROW_SPACER))
}

fn build_add_path_button(
    validator: ValidationFn<PathEntry, PathBuf, ArcStr>,
) -> impl Widget<Arc<Vec<PathValidationState>>> {
    Flex::row()
        .with_child(Icon::add().padding(3.0))
        .with_child(
            Label::new("Add path")
                .with_font(theme::font::HEADER_TWO)
                .with_line_break_mode(LineBreaking::Clip),
        )
        .must_fill_main_axis(true)
        .on_click(move |_, state: &mut Arc<Vec<PathValidationState>>, _| {
            Arc::make_mut(state).push(PathValidationState::new(
                PathEntry::default(),
                validator.clone(),
            ));
        })
        .background(theme::hot_or_active_painter(
            druid::theme::BUTTON_BORDER_RADIUS,
        ))
}

impl CompileTabState {
    pub fn new(includes: Vec<String>, files: Vec<String>) -> CompileTabState {
        CompileTabState {
            error: None,
            includes: Arc::new(
                includes
                    .into_iter()
                    .map(PathEntry::new)
                    .map(|include| ValidationState::new(include, VALIDATE_INCLUDE.clone()))
                    .collect(),
            ),
            files: Arc::new(
                files
                    .into_iter()
                    .map(PathEntry::new)
                    .map(|file| ValidationState::new(file, VALIDATE_FILE.clone()))
                    .collect(),
            ),
        }
    }

    pub fn serde_includes(&self) -> Vec<String> {
        self.includes
            .iter()
            .map(|d| d.text().path.to_string())
            .collect()
    }

    pub fn serde_files(&self) -> Vec<String> {
        self.files
            .iter()
            .map(|d| d.text().path.to_string())
            .collect()
    }

    pub fn includes(&self) -> impl Iterator<Item = PathBuf> + '_ {
        self.includes.iter().flat_map(|d| d.result()).cloned()
    }

    pub fn files(&self) -> impl Iterator<Item = PathBuf> + '_ {
        self.files.iter().flat_map(|d| d.result()).cloned()
    }

    pub fn is_valid(&self) -> bool {
        !self.includes.is_empty()
            && self.includes.iter().all(|d| d.is_valid())
            && !self.files.is_empty()
            && self.files.iter().all(|d| d.is_valid())
    }
}

impl Default for CompileTabState {
    fn default() -> Self {
        CompileTabState {
            error: None,
            includes: Arc::new(vec![ValidationState::new(
                PathEntry::default(),
                VALIDATE_INCLUDE.clone(),
            )]),
            files: Arc::new(vec![ValidationState::new(
                PathEntry::default(),
                VALIDATE_INCLUDE.clone(),
            )]),
        }
    }
}

impl PathEntry {
    fn new(path: String) -> Self {
        PathEntry {
            path: Arc::new(path),
            deleted: false,
        }
    }
}

static VALIDATE_INCLUDE: Lazy<ValidationFn<PathEntry, PathBuf, ArcStr>> =
    Lazy::new(|| Arc::new(validate_file));
static VALIDATE_FILE: Lazy<ValidationFn<PathEntry, PathBuf, ArcStr>> =
    Lazy::new(|| Arc::new(validate_include));

fn validate_include(entry: &PathEntry) -> Result<PathBuf, ArcStr> {
    Ok(PathBuf::from(entry.path.as_str()))
}

fn validate_file(entry: &PathEntry) -> Result<PathBuf, ArcStr> {
    Ok(PathBuf::from(entry.path.as_str()))
}

struct DeletePathController;

impl<W> Controller<Arc<Vec<PathValidationState>>, W> for DeletePathController
where
    W: Widget<Arc<Vec<PathValidationState>>>,
{
    fn event(
        &mut self,
        child: &mut W,
        ctx: &mut EventCtx,
        event: &Event,
        data: &mut Arc<Vec<PathValidationState>>,
        env: &Env,
    ) {
        match event {
            Event::Command(cmd) if cmd.is(DELETE_PATH) => {
                Arc::make_mut(data).retain(|e| !e.text().deleted);
            }
            _ => child.event(ctx, event, data, env),
        }
    }
}
