use std::borrow::Cow;
use std::io;
use std::iter;
use std::ops::Range;
use std::sync::Arc;

use druid::piet::{
    self, FontStyle, FontWeight, PietTextLayoutBuilder, TextAttribute, TextStorage as _,
};
use druid::Data;
use druid::{
    piet::TextLayoutBuilder,
    text::{EditableText, StringCursor, TextStorage},
};
use memchr::Memchr;
use once_cell::sync::Lazy;
use syntect::highlighting::{
    self, HighlightState, Highlighter, RangedHighlightIterator, Theme, ThemeSet,
};
use syntect::parsing::{ParseState, ScopeStack, SyntaxReference, SyntaxSet};

static SYNTAX_SET: Lazy<SyntaxSet> = Lazy::new(SyntaxSet::load_defaults_newlines);
static JSON_SYNTAX: Lazy<&'static SyntaxReference> =
    Lazy::new(|| SYNTAX_SET.find_syntax_by_token("json").unwrap());
static THEME: Lazy<Theme> = Lazy::new(|| {
    ThemeSet::load_from_reader(&mut io::Cursor::new(include_bytes!(
        "../assets/theme.tmTheme"
    )))
    .unwrap()
});
static THEME_HIGHLIGHTER: Lazy<Highlighter<'static>> = Lazy::new(|| Highlighter::new(&THEME));

#[derive(Debug, Clone)]
pub struct JsonText {
    data: String,
    styles: Arc<[(highlighting::Style, Range<usize>)]>,
}

fn get_styles(text: &str) -> Vec<(highlighting::Style, Range<usize>)> {
    let mut result = Vec::new();

    let mut highlight_state = HighlightState::new(&THEME_HIGHLIGHTER, ScopeStack::new());
    let mut parse_state = ParseState::new(&JSON_SYNTAX);

    for (start, line) in iter_lines(text) {
        let ops = parse_state.parse_line(line, &SYNTAX_SET);
        result.extend(
            RangedHighlightIterator::new(&mut highlight_state, &ops, line, &THEME_HIGHLIGHTER)
                .map(|(style, _, range)| (style, (start + range.start)..(start + range.end))),
        );
    }

    result
}

fn iter_lines<'a>(text: &'a str) -> impl Iterator<Item = (usize, &'a str)> + 'a {
    Memchr::new(b'\n', text.as_bytes())
        .map(|idx| idx + 1)
        .chain(iter::once(text.len()))
        .scan(0, move |start, end| {
            let range = *start..end;
            *start = end;
            Some((range.start, &text[range]))
        })
}

fn color(c: highlighting::Color) -> druid::Color {
    druid::Color::rgba8(c.r, c.g, c.b, c.a)
}

impl Default for JsonText {
    fn default() -> Self {
        JsonText::from_str(r#"{ "hello": "world", "foo": 1, "bar": null }"#)
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
        self.styles = get_styles(self.as_str()).into();
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
        JsonText::from(s.to_owned())
    }
}

impl piet::TextStorage for JsonText {
    fn as_str(&self) -> &str {
        self.data.as_str()
    }
}

impl From<String> for JsonText {
    fn from(s: String) -> Self {
        JsonText {
            styles: get_styles(&s).into(),
            data: s,
        }
    }
}
