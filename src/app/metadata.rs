use std::{str::FromStr, sync::Arc};

use base64::{
    alphabet,
    engine::{DecodePaddingMode, Engine, GeneralPurpose, GeneralPurposeConfig},
};
use druid::{
    widget::{
        prelude::*, Controller, CrossAxisAlignment, FillStrat, Flex, Label, LineBreaking, List,
        TextBox,
    },
    Lens, Point, Selector, WidgetExt, WidgetPod,
};
use once_cell::sync::Lazy;
use serde::{Deserialize, Serialize};
use tonic::metadata::{
    AsciiMetadataKey, AsciiMetadataValue, BinaryMetadataKey, BinaryMetadataValue, MetadataMap,
};

use crate::{
    theme::{self, BODY_SPACER},
    widget::{FinishEditController, FormField, Icon, ValidationFn, ValidationState},
};

pub type State = Arc<Vec<Entry>>;

type EntryValidation = ValidationState<EditableEntry, ParsedEntry, Arc<String>>;

#[derive(Debug, Default, Clone, Data, Lens)]
pub struct EditableState {
    entries: Arc<Vec<EntryValidation>>,
}

#[derive(Debug, Default, Clone, Data, Lens, Serialize, Deserialize)]
pub struct Entry {
    key: String,
    value: String,
}

#[derive(Debug, Default, Clone, Data, Lens, Serialize, Deserialize)]
pub struct EditableEntry {
    key: String,
    value: String,
    #[serde(skip)]
    deleted: bool,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ParsedEntry {
    Binary {
        key: BinaryMetadataKey,
        value: BinaryMetadataValue,
    },
    Ascii {
        key: AsciiMetadataKey,
        value: AsciiMetadataValue,
    },
}

const GRID_NARROW_SPACER: f64 = 2.0;

pub(in crate::app) fn build() -> impl Widget<State> {
    List::new(build_row).with_spacing(GRID_NARROW_SPACER)
}

pub(in crate::app) fn build_editable() -> impl Widget<EditableState> {
    let parent = WidgetId::next();
    EditableLayout {
        add_button: WidgetPod::new(build_add_button().boxed()),
        metadata: WidgetPod::new(
            List::new(move || build_editable_row(parent))
                .with_spacing(GRID_NARROW_SPACER)
                .lens(EditableState::entries)
                .controller(DeleteMetadataController)
                .with_id(parent)
                .scroll()
                .vertical()
                .boxed(),
        ),
    }
}

fn build_row() -> impl Widget<Entry> {
    Flex::row()
        .cross_axis_alignment(CrossAxisAlignment::Fill)
        .with_flex_child(
            theme::text_box_scope(TextBox::<String>::new().expand_width()).lens(Entry::key),
            0.33,
        )
        .with_spacer(GRID_NARROW_SPACER)
        .with_flex_child(
            theme::text_box_scope(TextBox::<String>::new().expand_width()).lens(Entry::value),
            0.67,
        )
}

fn build_editable_row(parent: WidgetId) -> impl Widget<EntryValidation> {
    let close = Icon::close()
        .with_fill(FillStrat::ScaleDown)
        .background(theme::hot_or_active_painter(
            druid::theme::BUTTON_BORDER_RADIUS,
        ))
        .on_click(move |ctx: &mut EventCtx, data: &mut EditableEntry, _| {
            data.deleted = true;
            ctx.submit_command(DELETE_METADATA.to(parent));
        });

    let form_id = WidgetId::next();
    FormField::new(
        form_id,
        Flex::row()
            .cross_axis_alignment(CrossAxisAlignment::Fill)
            .with_flex_child(
                theme::text_box_scope(TextBox::<String>::new().expand_width())
                    .controller(FinishEditController::new(form_id))
                    .lens(EditableEntry::key),
                0.33,
            )
            .with_spacer(GRID_NARROW_SPACER)
            .with_flex_child(
                theme::text_box_scope(TextBox::<String>::new().expand_width())
                    .controller(FinishEditController::new(form_id))
                    .lens(EditableEntry::value),
                0.67,
            )
            .with_spacer(GRID_NARROW_SPACER)
            .with_child(close),
    )
}

fn build_add_button() -> impl Widget<EditableState> {
    Flex::row()
        .with_child(Icon::add().padding(3.0))
        .with_child(
            Label::new("Add metadata")
                .with_font(theme::font::HEADER_TWO)
                .with_line_break_mode(LineBreaking::Clip),
        )
        .must_fill_main_axis(true)
        .on_click(|_, state: &mut EditableState, _| {
            Arc::make_mut(&mut state.entries).push(ValidationState::new(
                Default::default(),
                VALIDATE_ENTRY.clone(),
            ));
        })
        .background(theme::hot_or_active_painter(
            druid::theme::BUTTON_BORDER_RADIUS,
        ))
}

struct EditableLayout {
    metadata: WidgetPod<EditableState, Box<dyn Widget<EditableState>>>,
    add_button: WidgetPod<EditableState, Box<dyn Widget<EditableState>>>,
}

struct DeleteMetadataController;

pub const DELETE_METADATA: Selector = Selector::new("app.metadata.delete");

impl EditableState {
    pub fn metadata(&self) -> MetadataMap {
        let mut map = MetadataMap::new();
        for entry in self.entries.iter() {
            if let Ok(parsed_entry) = entry.result() {
                match parsed_entry.clone() {
                    ParsedEntry::Ascii { key, value } => {
                        map.append(key, value);
                    }
                    ParsedEntry::Binary { key, value } => {
                        map.append_bin(key, value);
                    }
                }
            }
        }
        map
    }
}

impl<W> Controller<EditableState, W> for DeleteMetadataController
where
    W: Widget<EditableState>,
{
    fn event(
        &mut self,
        child: &mut W,
        ctx: &mut EventCtx,
        event: &Event,
        data: &mut EditableState,
        env: &Env,
    ) {
        match event {
            Event::Command(cmd) if cmd.is(DELETE_METADATA) => {
                Arc::make_mut(&mut data.entries).retain(|e| !e.text().deleted);
            }
            _ => child.event(ctx, event, data, env),
        }
    }
}

impl Widget<EditableState> for EditableLayout {
    fn event(&mut self, ctx: &mut EventCtx, event: &Event, data: &mut EditableState, env: &Env) {
        self.metadata.event(ctx, event, data, env);
        self.add_button.event(ctx, event, data, env);
    }

    fn lifecycle(
        &mut self,
        ctx: &mut LifeCycleCtx,
        event: &LifeCycle,
        data: &EditableState,
        env: &Env,
    ) {
        self.metadata.lifecycle(ctx, event, data, env);
        self.add_button.lifecycle(ctx, event, data, env);
    }

    fn update(&mut self, ctx: &mut UpdateCtx, _: &EditableState, data: &EditableState, env: &Env) {
        self.metadata.update(ctx, data, env);
        self.add_button.update(ctx, data, env);
    }

    fn layout(
        &mut self,
        ctx: &mut LayoutCtx,
        bc: &BoxConstraints,
        data: &EditableState,
        env: &Env,
    ) -> Size {
        let body_spacer = if data.entries.is_empty() {
            0.0
        } else {
            BODY_SPACER
        };

        let width = bc.max().width;
        let max_height = (bc.max().height - GRID_NARROW_SPACER - body_spacer).max(bc.min().height);
        let tight_bc = BoxConstraints::new(
            Size::new(width, bc.min().height),
            Size::new(width, max_height),
        );

        let add_button_size = self.add_button.layout(ctx, &tight_bc, data, env);

        let metadata_bc = tight_bc.shrink_max_height_to(
            bc.max().height - add_button_size.height - GRID_NARROW_SPACER - body_spacer,
        );
        let metadata_size = self.metadata.layout(ctx, &metadata_bc, data, env);

        self.metadata.set_origin(ctx, Point::new(0.0, body_spacer));
        self.add_button.set_origin(
            ctx,
            Point::new(0.0, body_spacer + metadata_size.height + GRID_NARROW_SPACER),
        );

        bc.constrain(Size::new(
            width,
            body_spacer + add_button_size.height + GRID_NARROW_SPACER + metadata_size.height,
        ))
    }

    fn paint(&mut self, ctx: &mut PaintCtx, data: &EditableState, env: &Env) {
        self.metadata.paint(ctx, data, env);
        self.add_button.paint(ctx, data, env);
    }
}

impl Data for ParsedEntry {
    fn same(&self, other: &Self) -> bool {
        self == other
    }
}

static VALIDATE_ENTRY: Lazy<ValidationFn<EditableEntry, ParsedEntry, Arc<String>>> =
    Lazy::new(|| Arc::new(validate_entry));

fn validate_entry(raw: &EditableEntry) -> Result<ParsedEntry, Arc<String>> {
    if let Ok(key) = BinaryMetadataKey::from_str(&raw.key) {
        const STANDARD: GeneralPurpose = GeneralPurpose::new(
            &alphabet::STANDARD,
            GeneralPurposeConfig::new()
                .with_encode_padding(true)
                .with_decode_padding_mode(DecodePaddingMode::Indifferent),
        );

        let bytes = STANDARD
            .decode(&raw.value)
            .map_err(|_| Arc::new("invalid base64".to_owned()))?;

        let value =
            BinaryMetadataValue::try_from(bytes).map_err(|err| Arc::new(err.to_string()))?;
        Ok(ParsedEntry::Binary { key, value })
    } else {
        match AsciiMetadataKey::from_str(&raw.key) {
            Ok(key) => {
                let value = AsciiMetadataValue::try_from(&raw.value)
                    .map_err(|err| Arc::new(err.to_string()))?;
                Ok(ParsedEntry::Ascii { key, value })
            }
            Err(err) => Err(Arc::new(err.to_string())),
        }
    }
}