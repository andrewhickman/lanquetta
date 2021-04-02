pub mod serde;

mod highlight;

use std::borrow::Cow;
use std::io;
use std::ops::Range;
use std::sync::Arc;

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

impl JsonText {
    pub fn pretty(data: impl AsRef<str> + Into<String>) -> Self {
        let data = prettify(data.as_ref()).unwrap_or_else(|| data.into());
        JsonText {
            styles: highlight::get_styles(&data).into(),
            data: Arc::new(data),
        }
    }

    pub fn prettify(&mut self) {
        if let Some(pretty) = prettify(self.as_str()) {
            self.styles = highlight::get_styles(&pretty).into();
            self.data = Arc::new(pretty);
        }
    }

    pub fn plain_text(data: impl Into<Arc<String>>) -> Self {
        JsonText {
            data: data.into(),
            styles: Arc::new([]),
        }
    }

    fn original_data(&self) -> &str {
        &self.data
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
