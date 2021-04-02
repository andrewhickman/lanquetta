use std::{io, iter, ops::Range};

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
        "../../assets/theme.tmTheme"
    )))
    .unwrap()
});
static THEME_HIGHLIGHTER: Lazy<Highlighter<'static>> = Lazy::new(|| Highlighter::new(&THEME));

pub fn get_styles(text: &str) -> Vec<(highlighting::Style, Range<usize>)> {
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

fn iter_lines(text: &str) -> impl Iterator<Item = (usize, &str)> {
    Memchr::new(b'\n', text.as_bytes())
        .map(|idx| idx + 1)
        .chain(iter::once(text.len()))
        .scan(0, move |start, end| {
            let range = *start..end;
            *start = end;
            Some((range.start, &text[range]))
        })
}
