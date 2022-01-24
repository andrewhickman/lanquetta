use std::{ops::Range, str::Chars};

use druid::piet::TextAttribute;
use once_cell::sync::Lazy;
use regex::Regex;

use crate::theme::color;

pub fn get_styles(text: &str) -> Vec<(TextAttribute, Range<usize>)> {
    let mut result = Vec::new();
    Highlighter {
        text: text.chars(),
        styles: &mut result,
        pos: 0,
    }
    .highlight_document();

    result.sort_unstable_by_key(|(_, range)| range.start);
    result
}

struct Highlighter<'a> {
    text: Chars<'a>,
    pos: usize,
    styles: &'a mut Vec<(TextAttribute, Range<usize>)>,
}

impl<'a> Highlighter<'a> {
    fn highlight_document(&mut self) {
        self.highlight_value();
        self.skip_whitespace();
        while self.peek().is_some() {
            self.highlight_invalid();
        }
    }

    fn highlight_value(&mut self) {
        self.skip_whitespace();

        if let Some(ch) = self.peek() {
            match ch {
                '[' => self.highlight_array(),
                '{' => self.highlight_object(),
                'f' | 'n' | 't' => self.highlight_constant(),
                '-' | '0'..='9' => self.highlight_number(),
                '"' => self.highlight_string(false),
                _ => {
                    self.highlight_invalid();
                }
            }
        }
    }

    fn highlight_array(&mut self) {
        self.bump();
        self.skip_whitespace();

        let mut expecting_end = true;
        while let Some(ch) = self.peek() {
            match ch {
                ']' if expecting_end => {
                    self.bump();
                    break;
                }
                _ => {
                    self.highlight_value();
                    self.skip_whitespace();
                    expecting_end = !self.skip_char(',');
                    self.skip_whitespace();
                }
            }
        }
    }

    fn highlight_object(&mut self) {
        self.bump();
        self.skip_whitespace();

        let mut expecting_end = true;
        while let Some(ch) = self.peek() {
            match ch {
                '}' if expecting_end => {
                    self.bump();
                    break;
                }
                '"' => {
                    self.highlight_string(true);
                    self.skip_whitespace();
                    self.skip_char(':');
                    self.highlight_value();
                    self.skip_whitespace();
                    expecting_end = !self.skip_char(',');
                    self.skip_whitespace();
                }
                _ => {
                    self.highlight_invalid();
                }
            }
        }
    }

    fn highlight_constant(&mut self) {
        static CONSTANT: Lazy<Regex> = Lazy::new(|| Regex::new("^(?:false|true|null)").unwrap());

        if let Some(range) = self.skip_pattern(&CONSTANT) {
            self.styles.push((
                TextAttribute::TextColor(color::active(color::ACCENT, color::TEXT)),
                range,
            ));
        }
    }

    fn highlight_number(&mut self) {
        static NUMBER: Lazy<Regex> =
            Lazy::new(|| Regex::new(r#"^-?(?:0|[1-9]\d*)(?:\.\d+)?(?:[eE][+-]?\d+)?"#).unwrap());

        if let Some(range) = self.skip_pattern(&NUMBER) {
            self.styles
                .push((TextAttribute::TextColor(color::BOLD_ACCENT), range));
        }
    }

    fn highlight_string(&mut self, object_key: bool) {
        static STRING_ESCAPE: Lazy<Regex> =
            Lazy::new(|| Regex::new(r#"^\\(:?["\\/bfnrt]|u[0-9a-fA-F]{4})"#).unwrap());

        let start = self.pos();

        self.bump();

        while let Some(ch) = self.peek() {
            match ch {
                '"' => {
                    self.bump();
                    break;
                }
                '\\' => {
                    if self.skip_pattern(&STRING_ESCAPE).is_none() {
                        self.highlight_invalid();
                    }
                }
                _ => self.bump(),
            }
        }

        let end = self.pos();

        let color = if object_key {
            color::SUBTLE_ACCENT
        } else {
            color::active(color::BOLD_ACCENT, color::TEXT)
        };
        self.styles
            .push((TextAttribute::TextColor(color), start..end));
    }

    fn highlight_invalid(&mut self) {
        let start = self.pos();
        self.bump();
        let end = self.pos();

        match self.styles.last_mut() {
            Some((TextAttribute::TextColor(color::ERROR), range)) if range.end == start => {
                range.end = end;
            }
            _ => {
                self.styles
                    .push((TextAttribute::TextColor(color::ERROR), start..end));
            }
        }
    }

    fn skip_pattern(&mut self, pattern: &Regex) -> Option<Range<usize>> {
        if let Some(m) = pattern.find(self.text.as_str()) {
            debug_assert_eq!(m.start(), 0);
            Some(self.advance(m.end()))
        } else {
            self.highlight_invalid();
            None
        }
    }

    fn skip_whitespace(&mut self) {
        while let Some(ch) = self.peek() {
            if ch.is_ascii_whitespace() {
                self.bump();
            } else {
                break;
            }
        }
    }

    fn skip_char(&mut self, expected: char) -> bool {
        if let Some(ch) = self.peek() {
            if ch == expected {
                self.bump();
                return true;
            }
        }
        false
    }

    fn peek(&mut self) -> Option<char> {
        self.text.as_str().chars().next()
    }

    fn bump(&mut self) {
        let ch = self
            .text
            .next()
            .expect("bump called without peek returning Some");
        self.pos += ch.len_utf8();
    }

    fn pos(&mut self) -> usize {
        self.pos
    }

    fn advance(&mut self, n: usize) -> Range<usize> {
        let start = self.pos;
        self.pos += n;
        let end = self.pos;

        self.text = self.text.as_str()[n..].chars();

        start..end
    }
}
