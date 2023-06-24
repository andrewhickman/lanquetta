use std::{borrow::Cow, ffi::OsString, fmt, fs, ops::Range, path::PathBuf, sync::Arc};

use druid::{
    piet::{PietTextLayoutBuilder, TextStorage},
    text::{EditableText, EnvUpdateCtx, Link, StringCursor},
    widget::{prelude::*, Controller, CrossAxisAlignment, Flex, Label, Scroll},
    ArcStr, Data, FileDialogOptions, FileInfo, Lens, Selector, WidgetExt,
};
use once_cell::sync::Lazy;
use serde::{Deserialize, Serialize};

use crate::{
    app::command,
    lens,
    theme::{self, BODY_PADDING, GRID_NARROW_SPACER},
    widget::{error_label, input, EditableList, FormField, ValidationFn, ValidationState},
};

#[derive(Default, Debug, Clone, Data, Lens)]
pub struct CompileTabState {
    error: Option<ArcStr>,
    #[lens(name = "includes_lens")]
    includes: Arc<Vec<PathValidationState>>,
}

#[derive(Default, Debug, Clone, Data, Serialize, Deserialize)]
pub struct CompileOptions {
    includes: Arc<Vec<PathBuf>>,
}

type PathValidationState = ValidationState<PathEntry, PathBuf, ArcStr>;

const ADD_PATH: Selector<Vec<FileInfo>> = Selector::new("app.body.compile.add-path");
const DELETE_PATH: Selector = Selector::new("app.body.compile.delete-path");

#[derive(Default, Debug, Clone, Data, Lens)]
struct PathEntry {
    path: PathText,
    deleted: bool,
}

pub fn build_body() -> impl Widget<CompileTabState> {
    let parent = WidgetId::next();

    Scroll::new(
        Flex::column()
            .with_child(Label::new("Include paths").with_font(theme::font::HEADER_TWO))
            .with_spacer(theme::BODY_SPACER)
            .with_child(
                EditableList::new("Add path", add_path, build_path_row)
                    .lens(CompileTabState::includes_lens),
            )
            .cross_axis_alignment(CrossAxisAlignment::Start)
            .padding(BODY_PADDING),
    )
    .vertical()
    .expand_height()
    .controller(CompileTabController)
    .with_id(parent)
}

fn add_path(ctx: &mut EventCtx, _: &mut Arc<Vec<PathValidationState>>, _: &Env) {
    ctx.submit_command(
        druid::commands::SHOW_OPEN_PANEL.with(
            FileDialogOptions::new()
                .accept_multiple_command(ADD_PATH)
                .select_directories()
                .multi_selection()
                .title("Add include paths")
                .button_text("Add"),
        ),
    );
}

fn build_path_row() -> impl Widget<PathValidationState> {
    let form_field = FormField::text_box(input(path_placeholder_text()).lens(PathEntry::path));

    let error = error_label((GRID_NARROW_SPACER, 0.0, 0.0, 0.0)).lens(lens::Project::new(
        |path: &PathValidationState| path.display_error(),
    ));

    Flex::row()
        .cross_axis_alignment(CrossAxisAlignment::Fill)
        .with_flex_child(form_field, 1.0)
        .with_child(error)
        .padding((0.0, 0.0, 0.0, GRID_NARROW_SPACER))
}

fn path_placeholder_text() -> String {
    if cfg!(windows) {
        "C:\\path\\to\\include".to_owned()
    } else {
        "/path/to/include".to_owned()
    }
}

impl CompileTabState {
    pub fn new(options: &CompileOptions) -> CompileTabState {
        CompileTabState {
            error: None,
            includes: Arc::new(
                options
                    .includes
                    .iter()
                    .map(|include| {
                        ValidationState::new(
                            PathEntry::new(include.into()),
                            VALIDATE_INCLUDE.clone(),
                        )
                    })
                    .collect(),
            ),
        }
    }

    pub fn compile_options(&self) -> CompileOptions {
        CompileOptions {
            includes: Arc::new(
                self.includes
                    .iter()
                    .map(|d| PathBuf::from(d.text().path.raw.as_os_str()))
                    .collect(),
            ),
        }
    }
}

impl CompileOptions {
    pub fn includes(&self) -> &[PathBuf] {
        self.includes.as_slice()
    }
}

impl PathEntry {
    fn new(path: OsString) -> Self {
        PathEntry {
            path: PathText::new(path),
            deleted: false,
        }
    }
}

static VALIDATE_INCLUDE: Lazy<ValidationFn<PathEntry, PathBuf, ArcStr>> =
    Lazy::new(|| Arc::new(validate_include));

fn validate_include(entry: &PathEntry) -> Result<PathBuf, ArcStr> {
    match fs::metadata(entry.path.raw.as_os_str()) {
        Ok(metadata) => {
            if metadata.is_dir() {
                Ok(PathBuf::from(entry.path.raw.as_os_str()))
            } else {
                Err("not a directory".into())
            }
        }
        Err(err) => Err(format!("failed to read metadata: {}", err).into()),
    }
}

struct CompileTabController;

impl<W> Controller<CompileTabState, W> for CompileTabController
where
    W: Widget<CompileTabState>,
{
    fn event(
        &mut self,
        child: &mut W,
        ctx: &mut EventCtx,
        event: &Event,
        data: &mut CompileTabState,
        env: &Env,
    ) {
        match event {
            Event::Command(cmd) if cmd.is(DELETE_PATH) => {
                Arc::make_mut(&mut data.includes).retain(|e| !e.text().deleted);
            }
            Event::Command(cmd) if cmd.is(ADD_PATH) => {
                let paths = cmd.get_unchecked(ADD_PATH);

                for path in paths {
                    Arc::make_mut(&mut data.includes).push(ValidationState::new(
                        PathEntry::new(path.path.as_os_str().to_owned()),
                        VALIDATE_INCLUDE.clone(),
                    ))
                }
            }
            _ => child.event(ctx, event, data, env),
        }
    }

    fn update(
        &mut self,
        child: &mut W,
        ctx: &mut UpdateCtx,
        old_data: &CompileTabState,
        data: &CompileTabState,
        env: &Env,
    ) {
        if !old_data.same(data) {
            ctx.submit_command(command::SET_COMPILE_OPTIONS.with(data.compile_options()));
        }

        child.update(ctx, old_data, data, env)
    }
}

/// A string backed by an OsString which preserves invalid UTF-8 until overwritten by modifications.
#[derive(Clone)]
struct PathText {
    raw: Arc<OsString>,
    display: Arc<String>,
}

impl Data for PathText {
    fn same(&self, other: &Self) -> bool {
        self.raw.same(&other.raw)
    }
}

impl TextStorage for PathText {
    fn as_str(&self) -> &str {
        self.display.as_str()
    }
}

impl druid::text::TextStorage for PathText {
    fn add_attributes(&self, builder: PietTextLayoutBuilder, env: &Env) -> PietTextLayoutBuilder {
        self.display.add_attributes(builder, env)
    }

    fn env_update(&self, ctx: &EnvUpdateCtx) -> bool {
        self.display.env_update(ctx)
    }

    fn links(&self) -> &[Link] {
        self.display.links()
    }
}

impl fmt::Debug for PathText {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.raw.fmt(f)
    }
}

impl Default for PathText {
    fn default() -> Self {
        PathText::new(OsString::new())
    }
}

impl PathText {
    fn new(raw: OsString) -> Self {
        PathText {
            display: Arc::new(raw.to_string_lossy().into_owned()),
            raw: Arc::new(raw),
        }
    }

    fn with_string_mut(&mut self, f: impl FnOnce(&mut String)) {
        f(Arc::make_mut(&mut self.display));
        if self.display.as_ref() != self.raw.to_string_lossy().as_ref() {
            let raw = Arc::make_mut(&mut self.raw);
            raw.clear();
            raw.push(self.display.as_str());
        }
    }
}

impl EditableText for PathText {
    fn cursor(&self, position: usize) -> Option<StringCursor> {
        self.display.cursor(position)
    }

    fn edit(&mut self, range: Range<usize>, new: impl Into<String>) {
        self.with_string_mut(|s| s.edit(range, new))
    }

    fn slice(&self, range: Range<usize>) -> Option<Cow<str>> {
        self.display.slice(range)
    }

    fn len(&self) -> usize {
        self.display.len()
    }

    fn prev_word_offset(&self, offset: usize) -> Option<usize> {
        self.display.prev_word_offset(offset)
    }

    fn next_word_offset(&self, offset: usize) -> Option<usize> {
        self.display.next_word_offset(offset)
    }

    fn prev_grapheme_offset(&self, offset: usize) -> Option<usize> {
        self.display.prev_grapheme_offset(offset)
    }

    fn next_grapheme_offset(&self, offset: usize) -> Option<usize> {
        self.display.next_grapheme_offset(offset)
    }

    fn prev_codepoint_offset(&self, offset: usize) -> Option<usize> {
        self.display.prev_codepoint_offset(offset)
    }

    fn next_codepoint_offset(&self, offset: usize) -> Option<usize> {
        self.display.next_codepoint_offset(offset)
    }

    fn preceding_line_break(&self, offset: usize) -> usize {
        self.display.preceding_line_break(offset)
    }

    fn next_line_break(&self, offset: usize) -> usize {
        self.display.next_line_break(offset)
    }

    fn is_empty(&self) -> bool {
        self.display.is_empty()
    }

    fn from_str(s: &str) -> Self {
        PathText::new(OsString::from(s))
    }
}
