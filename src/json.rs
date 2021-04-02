pub mod serde;

mod count;
mod exclude;
mod highlight;

use std::borrow::Cow;
use std::io;
use std::ops::Range;
use std::sync::Arc;

use anyhow::Result;
use druid::{
    piet::{
        self, FontStyle, FontWeight, PietTextLayoutBuilder, TextAttribute, TextLayoutBuilder,
        TextStorage as _,
    },
    text::{EditableText, StringCursor, TextStorage},
    Data,
};
use syntect::highlighting;

#[derive(Debug, Clone)]
pub struct JsonText {
    // Original data, present if this JSON has been shortened.
    original_data: Option<Arc<String>>,
    data: Arc<String>,
    styles: Arc<[(highlighting::Style, Range<usize>)]>,
}

fn color(c: highlighting::Color) -> druid::Color {
    druid::Color::rgba8(c.r, c.g, c.b, c.a)
}

fn prettify(s: &str) -> Option<String> {
    let mut result = io::Cursor::new(Vec::with_capacity(s.len()));
    let mut deserializer = serde_json::Deserializer::from_str(s);
    serde_transcode::transcode(
        &mut deserializer,
        &mut serde_json::Serializer::pretty(&mut result),
    )
    .ok()?;
    deserializer.end().ok()?;
    Some(String::from_utf8(result.into_inner()).unwrap())
}

#[derive(Copy, Clone, Debug)]
pub struct ShortenOptions {
    /// The maximum number of lines a JSON value can take up when printed.
    max_length: Option<u32>,
    /// The maximum depth to which a JSON value should be printed.
    max_depth: Option<u32>,
}

fn shorten(s: &str, opts: ShortenOptions) -> Option<String> {
    let excludes = count::count(opts, |ser| serialize(s, ser)).ok()?;
    let mut result = Vec::new();
    exclude::write(excludes, &mut result, |ser| serialize(s, ser)).ok()?;
    Some(String::from_utf8(result).unwrap())
}

fn serialize<S>(s: &str, ser: S) -> Result<()>
where
    S: ::serde::Serializer<Ok = (), Error = serde_json::Error>,
{
    let mut de = serde_json::Deserializer::from_str(s);
    serde_transcode::transcode(&mut de, ser)?;
    Ok(())
}

impl JsonText {
    pub fn pretty(data: impl AsRef<str> + Into<String>) -> Self {
        let data = prettify(data.as_ref()).unwrap_or_else(|| data.into());
        JsonText {
            original_data: None,
            styles: highlight::get_styles(&data).into(),
            data: Arc::new(data),
        }
    }

    pub fn prettify(&mut self) {
        if let Some(pretty) = prettify(self.as_str()) {
            *self = JsonText::pretty(pretty);
        }
    }

    pub fn short(data: impl Into<Arc<String>>) -> Self {
        let original_data = data.into();

        let options = ShortenOptions {
            max_length: Some(512),
            max_depth: Some(12),
        };

        let data = match shorten(&original_data, options) {
            Some(data) => data.into(),
            None => original_data.clone(),
        };

        JsonText {
            original_data: Some(original_data),
            styles: highlight::get_styles(&data).into(),
            data,
        }
    }

    pub fn plain_text(data: impl Into<Arc<String>>) -> Self {
        let data = data.into();
        JsonText {
            original_data: None,
            data: data.into(),
            styles: Arc::new([]),
        }
    }

    pub fn original_data(&self) -> &str {
        match &self.original_data {
            Some(data) => &data,
            None => &self.data,
        }
    }
}

impl Default for JsonText {
    fn default() -> Self {
        JsonText::from_str("")
    }
}

impl Data for JsonText {
    fn same(&self, other: &Self) -> bool {
        self.data.same(&other.data)
    }
}

impl TextStorage for JsonText {
    fn add_attributes(
        &self,
        mut builder: PietTextLayoutBuilder,
        _env: &druid::Env,
    ) -> PietTextLayoutBuilder {
        for (ref style, ref range) in self.styles.iter() {
            builder = builder.range_attribute(
                range.clone(),
                TextAttribute::TextColor(color(style.foreground)),
            );

            if style.font_style.contains(highlighting::FontStyle::BOLD) {
                builder =
                    builder.range_attribute(range.clone(), TextAttribute::Weight(FontWeight::BOLD));
            }

            if style.font_style.contains(highlighting::FontStyle::ITALIC) {
                builder =
                    builder.range_attribute(range.clone(), TextAttribute::Style(FontStyle::Italic));
            }

            if style
                .font_style
                .contains(highlighting::FontStyle::UNDERLINE)
            {
                builder = builder.range_attribute(range.clone(), TextAttribute::Underline(true));
            }
        }
        builder
    }
}

impl EditableText for JsonText {
    fn cursor(&self, position: usize) -> Option<StringCursor> {
        self.data.cursor(position)
    }

    fn edit(&mut self, range: Range<usize>, new: impl Into<String>) {
        self.data.edit(range, new);
        self.styles = highlight::get_styles(self.as_str()).into();
    }

    fn slice(&self, range: Range<usize>) -> Option<Cow<str>> {
        self.data.slice(range)
    }

    fn len(&self) -> usize {
        self.data.len()
    }

    fn prev_word_offset(&self, offset: usize) -> Option<usize> {
        self.data.prev_word_offset(offset)
    }

    fn next_word_offset(&self, offset: usize) -> Option<usize> {
        self.data.next_word_offset(offset)
    }

    fn prev_grapheme_offset(&self, offset: usize) -> Option<usize> {
        self.data.prev_grapheme_offset(offset)
    }

    fn next_grapheme_offset(&self, offset: usize) -> Option<usize> {
        self.data.next_grapheme_offset(offset)
    }

    fn prev_codepoint_offset(&self, offset: usize) -> Option<usize> {
        self.data.prev_codepoint_offset(offset)
    }

    fn next_codepoint_offset(&self, offset: usize) -> Option<usize> {
        self.data.next_codepoint_offset(offset)
    }

    fn preceding_line_break(&self, offset: usize) -> usize {
        self.data.preceding_line_break(offset)
    }

    fn next_line_break(&self, offset: usize) -> usize {
        self.data.next_line_break(offset)
    }

    fn is_empty(&self) -> bool {
        self.data.is_empty()
    }

    fn from_str(s: &str) -> Self {
        JsonText::pretty(s.to_owned())
    }
}

impl piet::TextStorage for JsonText {
    fn as_str(&self) -> &str {
        self.data.as_str()
    }
}
