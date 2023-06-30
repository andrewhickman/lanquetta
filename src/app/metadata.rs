use std::{str::FromStr, sync::Arc};

use base64::{
    alphabet,
    engine::{DecodePaddingMode, Engine, GeneralPurpose, GeneralPurposeConfig},
};
use druid::{
    widget::{prelude::*, CrossAxisAlignment, Flex, List},
    ArcStr, Insets, Lens, Point, WidgetExt, WidgetPod,
};
use once_cell::sync::Lazy;
use serde::{Deserialize, Serialize};
use tonic::metadata::{
    AsciiMetadataKey, AsciiMetadataValue, BinaryMetadataKey, BinaryMetadataValue, MetadataMap,
};

use crate::widget::{
    env_error_label, input, readonly_input, EditableList, FinishEditController, FormField,
    ValidationFn, ValidationState, ERROR_MESSAGE,
};

pub type State = Arc<Vec<Entry>>;

type EntryValidationState = ValidationState<EditableEntry, ParsedEntry>;

#[derive(Debug, Default, Clone, Data, Lens)]
pub struct EditableState {
    entries: Arc<Vec<EntryValidationState>>,
}

#[derive(Debug, Default, Clone, Data, Lens, Serialize, Deserialize)]
pub struct Entry {
    key: Arc<String>,
    value: Arc<String>,
}

#[derive(Debug, Default, Clone, Data)]
pub struct EditableEntry {
    key: Arc<String>,
    value: Arc<String>,
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
    EditableList::new(
        "Add metadata",
        |_, data, _| {
            Arc::make_mut(data).push(ValidationState::new(
                EditableEntry::default(),
                VALIDATE_ENTRY.clone(),
            ))
        },
        build_editable_row,
    )
    .lens(EditableState::entries)
}

fn build_row() -> impl Widget<Entry> {
    Flex::row()
        .cross_axis_alignment(CrossAxisAlignment::Fill)
        .with_flex_child(readonly_input().lens(Entry::key), 0.33)
        .with_spacer(GRID_NARROW_SPACER)
        .with_flex_child(readonly_input().lens(Entry::value), 0.67)
}

fn build_editable_row() -> impl Widget<EntryValidationState> {
    let form_id = WidgetId::next();
    FormField::new(
        form_id,
        EditableRowLayout {
            key: WidgetPod::new(
                input("key")
                    .controller(FinishEditController::new(form_id))
                    .boxed(),
            ),
            value: WidgetPod::new(
                input("value")
                    .controller(FinishEditController::new(form_id))
                    .boxed(),
            ),
            error: WidgetPod::new(env_error_label(Insets::ZERO).boxed()),
        },
    )
}

struct EditableLayout {
    metadata: WidgetPod<EditableState, Box<dyn Widget<EditableState>>>,
    add_button: WidgetPod<EditableState, Box<dyn Widget<EditableState>>>,
}

struct EditableRowLayout {
    key: WidgetPod<Arc<String>, Box<dyn Widget<Arc<String>>>>,
    value: WidgetPod<Arc<String>, Box<dyn Widget<Arc<String>>>>,
    error: WidgetPod<(), Box<dyn Widget<()>>>,
}

pub fn state_from_tonic(metadata: MetadataMap) -> State {
    Arc::new(
        metadata
            .iter()
            .map(|entry| match entry {
                tonic::metadata::KeyAndValueRef::Ascii(key, value) => Entry {
                    key: Arc::new(key.to_string()),
                    value: Arc::new(String::from_utf8_lossy(value.as_encoded_bytes()).into_owned()),
                },
                tonic::metadata::KeyAndValueRef::Binary(key, value) => Entry {
                    key: Arc::new(key.to_string()),
                    value: Arc::new(String::from_utf8_lossy(value.as_encoded_bytes()).into_owned()),
                },
            })
            .collect(),
    )
}

impl EditableState {
    pub fn new(mut metadata: State) -> EditableState {
        let entries = Arc::new(
            Arc::make_mut(&mut metadata)
                .drain(..)
                .map(|entry| {
                    ValidationState::new(
                        EditableEntry {
                            key: entry.key,
                            value: entry.value,
                            deleted: false,
                        },
                        VALIDATE_ENTRY.clone(),
                    )
                })
                .collect(),
        );
        EditableState { entries }
    }

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

    pub fn to_state(&self) -> State {
        Arc::new(
            self.entries
                .iter()
                .map(|e| Entry {
                    key: e.text().key.clone(),
                    value: e.text().value.clone(),
                })
                .collect(),
        )
    }

    pub fn is_empty(&self) -> bool {
        self.entries.is_empty()
    }

    pub fn is_valid(&self) -> bool {
        self.entries.iter().all(|e| e.is_valid())
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
        let width = bc.max().width;
        let max_height = (bc.max().height - GRID_NARROW_SPACER).max(bc.min().height);
        let tight_bc = BoxConstraints::new(
            Size::new(width, bc.min().height),
            Size::new(width, max_height),
        );

        let add_button_size = self.add_button.layout(ctx, &tight_bc, data, env);

        let metadata_bc = tight_bc
            .shrink_max_height_to(bc.max().height - add_button_size.height - GRID_NARROW_SPACER);
        let metadata_size = self.metadata.layout(ctx, &metadata_bc, data, env);

        self.metadata.set_origin(ctx, Point::ORIGIN);
        self.add_button.set_origin(
            ctx,
            Point::new(0.0, metadata_size.height + GRID_NARROW_SPACER),
        );

        bc.constrain(Size::new(
            width,
            add_button_size.height + GRID_NARROW_SPACER + metadata_size.height,
        ))
    }

    fn paint(&mut self, ctx: &mut PaintCtx, data: &EditableState, env: &Env) {
        self.metadata.paint(ctx, data, env);
        self.add_button.paint(ctx, data, env);
    }
}

impl Widget<EditableEntry> for EditableRowLayout {
    fn event(&mut self, ctx: &mut EventCtx, event: &Event, data: &mut EditableEntry, env: &Env) {
        self.key.event(ctx, event, &mut data.key, env);
        self.value.event(ctx, event, &mut data.value, env);
        if event.should_propagate_to_hidden() || env.try_get(ERROR_MESSAGE).is_ok() {
            self.error.event(ctx, event, &mut (), env);
        }
    }

    fn lifecycle(
        &mut self,
        ctx: &mut LifeCycleCtx,
        event: &LifeCycle,
        data: &EditableEntry,
        env: &Env,
    ) {
        self.key.lifecycle(ctx, event, &data.key, env);
        self.value.lifecycle(ctx, event, &data.value, env);
        if event.should_propagate_to_hidden() || env.try_get(ERROR_MESSAGE).is_ok() {
            self.error.lifecycle(ctx, event, &(), env);
        }
    }

    fn update(&mut self, ctx: &mut UpdateCtx, _: &EditableEntry, data: &EditableEntry, env: &Env) {
        self.key.update(ctx, &data.key, env);
        self.value.update(ctx, &data.value, env);
        if ctx.env_key_changed(&ERROR_MESSAGE) {
            self.error.update(ctx, &(), env);
        }
    }

    fn layout(
        &mut self,
        ctx: &mut LayoutCtx,
        bc: &BoxConstraints,
        data: &EditableEntry,
        env: &Env,
    ) -> Size {
        let key_width = ((bc.max().width - GRID_NARROW_SPACER * 2.0).max(0.0) / 3.0).floor();

        let key_bc = BoxConstraints::new(
            Size::new(key_width, bc.min().height),
            Size::new(key_width, bc.max().height),
        );
        let key_size = self.key.layout(ctx, &key_bc, &data.key, env);
        self.key.set_origin(ctx, Point::ZERO);

        let remaining_width = bc.max().width - GRID_NARROW_SPACER - key_size.width;

        let value_bc = if env.try_get(ERROR_MESSAGE).is_ok() {
            let error_bc = BoxConstraints::new(
                Size::new(100.0, key_size.height),
                Size::new(remaining_width - GRID_NARROW_SPACER, key_size.height),
            );
            let error_size = self.error.layout(ctx, &error_bc, &(), env);
            self.error
                .set_origin(ctx, Point::new(bc.max().width - error_size.width, 0.0));

            BoxConstraints::tight(Size::new(
                remaining_width - GRID_NARROW_SPACER - error_size.width,
                key_size.height,
            ))
        } else {
            BoxConstraints::tight(Size::new(remaining_width, key_size.height))
        };

        self.value.layout(ctx, &value_bc, &data.value, env);
        self.value
            .set_origin(ctx, Point::new(key_size.width + GRID_NARROW_SPACER, 0.0));

        Size::new(bc.max().width, key_size.height)
    }

    fn paint(&mut self, ctx: &mut PaintCtx, data: &EditableEntry, env: &Env) {
        self.key.paint(ctx, &data.key, env);
        self.value.paint(ctx, &data.value, env);
        if env.try_get(ERROR_MESSAGE).is_ok() {
            self.error.paint(ctx, &(), env);
        }
    }
}

impl Data for ParsedEntry {
    fn same(&self, other: &Self) -> bool {
        self == other
    }
}

static VALIDATE_ENTRY: Lazy<ValidationFn<EditableEntry, ParsedEntry>> =
    Lazy::new(|| Arc::new(validate_entry));

fn validate_entry(raw: &EditableEntry) -> Result<ParsedEntry, ArcStr> {
    if let Ok(key) = BinaryMetadataKey::from_str(&raw.key) {
        const STANDARD: GeneralPurpose = GeneralPurpose::new(
            &alphabet::STANDARD,
            GeneralPurposeConfig::new()
                .with_encode_padding(true)
                .with_decode_padding_mode(DecodePaddingMode::Indifferent),
        );

        let bytes = STANDARD
            .decode(raw.value.as_ref())
            .map_err(|_| "invalid base64")?;

        let value = BinaryMetadataValue::try_from(bytes).map_err(|err| err.to_string())?;
        Ok(ParsedEntry::Binary { key, value })
    } else if let Ok(key) = AsciiMetadataKey::from_str(&raw.key) {
        let value =
            AsciiMetadataValue::try_from(raw.value.as_ref()).map_err(|_| "invalid ascii")?;
        Ok(ParsedEntry::Ascii { key, value })
    } else {
        Err("invalid key".into())
    }
}
