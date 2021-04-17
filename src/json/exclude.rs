use std::collections::BTreeMap;
use std::io::{self, Write};
use std::ops::Range;

use anyhow::Result;
use serde_json::ser::{CharEscape, Formatter, PrettyFormatter};

pub fn write<F, W>(excludes: ExcludeSet, writer: W, f: F) -> Result<()>
where
    F: FnOnce(&mut serde_json::Serializer<W, Excluder>) -> Result<()>,
    W: Write,
{
    let excluder = Excluder {
        excludes,
        position: 0,
        depth: 0,
        pretty: PrettyFormatter::new(),
    };
    let mut ser = serde_json::Serializer::with_formatter(writer, excluder);
    f(&mut ser)
}

pub struct Excluder {
    excludes: ExcludeSet,
    position: u32,
    depth: u32,
    pretty: PrettyFormatter<'static>,
}

impl Excluder {
    fn writing(&self) -> bool {
        self.depth == 0
    }

    fn begin<W: ?Sized>(&mut self, writer: &mut W) -> io::Result<()>
    where
        W: Write,
    {
        if let Some(length) = self.excludes.get(self.position) {
            if self.writing() {
                write!(writer, " {} items... ", length)?
            }
            self.depth += 1;
        }
        self.position += 1;
        Ok(())
    }

    fn end(&mut self) {
        if self.excludes.get(self.position).is_some() {
            self.depth -= 1;
        }
        self.position += 1;
    }

    fn delegate(
        &mut self,
        f: impl FnOnce(&mut PrettyFormatter) -> io::Result<()>,
    ) -> io::Result<()> {
        if self.writing() {
            f(&mut self.pretty)
        } else {
            Ok(())
        }
    }
}

impl Formatter for Excluder {
    fn write_null<W: ?Sized>(&mut self, writer: &mut W) -> io::Result<()>
    where
        W: Write,
    {
        self.delegate(|f| f.write_null(writer))
    }

    fn write_bool<W: ?Sized>(&mut self, writer: &mut W, value: bool) -> io::Result<()>
    where
        W: Write,
    {
        self.delegate(|f| f.write_bool(writer, value))
    }

    fn write_i8<W: ?Sized>(&mut self, writer: &mut W, value: i8) -> io::Result<()>
    where
        W: Write,
    {
        self.delegate(|f| f.write_i8(writer, value))
    }

    fn write_i16<W: ?Sized>(&mut self, writer: &mut W, value: i16) -> io::Result<()>
    where
        W: Write,
    {
        self.delegate(|f| f.write_i16(writer, value))
    }

    fn write_i32<W: ?Sized>(&mut self, writer: &mut W, value: i32) -> io::Result<()>
    where
        W: Write,
    {
        self.delegate(|f| f.write_i32(writer, value))
    }

    fn write_i64<W: ?Sized>(&mut self, writer: &mut W, value: i64) -> io::Result<()>
    where
        W: Write,
    {
        self.delegate(|f| f.write_i64(writer, value))
    }

    fn write_u8<W: ?Sized>(&mut self, writer: &mut W, value: u8) -> io::Result<()>
    where
        W: Write,
    {
        self.delegate(|f| f.write_u8(writer, value))
    }

    fn write_u16<W: ?Sized>(&mut self, writer: &mut W, value: u16) -> io::Result<()>
    where
        W: Write,
    {
        self.delegate(|f| f.write_u16(writer, value))
    }

    fn write_u32<W: ?Sized>(&mut self, writer: &mut W, value: u32) -> io::Result<()>
    where
        W: Write,
    {
        self.delegate(|f| f.write_u32(writer, value))
    }

    fn write_u64<W: ?Sized>(&mut self, writer: &mut W, value: u64) -> io::Result<()>
    where
        W: Write,
    {
        self.delegate(|f| f.write_u64(writer, value))
    }

    fn write_f32<W: ?Sized>(&mut self, writer: &mut W, value: f32) -> io::Result<()>
    where
        W: Write,
    {
        self.delegate(|f| f.write_f32(writer, value))
    }

    fn write_f64<W: ?Sized>(&mut self, writer: &mut W, value: f64) -> io::Result<()>
    where
        W: Write,
    {
        self.delegate(|f| f.write_f64(writer, value))
    }

    fn write_number_str<W: ?Sized>(&mut self, writer: &mut W, value: &str) -> io::Result<()>
    where
        W: Write,
    {
        self.delegate(|f| f.write_number_str(writer, value))
    }

    fn begin_string<W: ?Sized>(&mut self, writer: &mut W) -> io::Result<()>
    where
        W: Write,
    {
        self.delegate(|f| f.begin_string(writer))
    }

    fn end_string<W: ?Sized>(&mut self, writer: &mut W) -> io::Result<()>
    where
        W: Write,
    {
        self.delegate(|f| f.end_string(writer))
    }

    fn write_string_fragment<W: ?Sized>(&mut self, writer: &mut W, fragment: &str) -> io::Result<()>
    where
        W: Write,
    {
        self.delegate(|f| f.write_string_fragment(writer, fragment))
    }

    fn write_char_escape<W: ?Sized>(
        &mut self,
        writer: &mut W,
        char_escape: CharEscape,
    ) -> io::Result<()>
    where
        W: Write,
    {
        self.delegate(|f| f.write_char_escape(writer, char_escape))
    }

    fn begin_array<W: ?Sized>(&mut self, writer: &mut W) -> io::Result<()>
    where
        W: Write,
    {
        self.delegate(|f| f.begin_array(writer))?;
        self.begin(writer)
    }

    fn end_array<W: ?Sized>(&mut self, writer: &mut W) -> io::Result<()>
    where
        W: Write,
    {
        self.end();
        self.delegate(|f| f.end_array(writer))
    }

    fn begin_array_value<W: ?Sized>(&mut self, writer: &mut W, first: bool) -> io::Result<()>
    where
        W: Write,
    {
        self.delegate(|f| f.begin_array_value(writer, first))
    }

    fn end_array_value<W: ?Sized>(&mut self, writer: &mut W) -> io::Result<()>
    where
        W: Write,
    {
        self.delegate(|f| f.end_array_value(writer))
    }

    fn begin_object<W: ?Sized>(&mut self, writer: &mut W) -> io::Result<()>
    where
        W: Write,
    {
        self.delegate(|f| f.begin_object(writer))?;
        self.begin(writer)
    }

    fn end_object<W: ?Sized>(&mut self, writer: &mut W) -> io::Result<()>
    where
        W: Write,
    {
        self.end();
        self.delegate(|f| f.end_object(writer))
    }

    fn begin_object_key<W: ?Sized>(&mut self, writer: &mut W, first: bool) -> io::Result<()>
    where
        W: Write,
    {
        self.delegate(|f| f.begin_object_key(writer, first))
    }

    fn end_object_key<W: ?Sized>(&mut self, writer: &mut W) -> io::Result<()>
    where
        W: Write,
    {
        self.delegate(|f| f.end_object_key(writer))
    }

    fn begin_object_value<W: ?Sized>(&mut self, writer: &mut W) -> io::Result<()>
    where
        W: Write,
    {
        self.delegate(|f| f.begin_object_value(writer))
    }

    fn end_object_value<W: ?Sized>(&mut self, writer: &mut W) -> io::Result<()>
    where
        W: Write,
    {
        self.delegate(|f| f.end_object_value(writer))
    }

    fn write_raw_fragment<W: ?Sized>(&mut self, writer: &mut W, fragment: &str) -> io::Result<()>
    where
        W: Write,
    {
        self.delegate(|f| f.write_raw_fragment(writer, fragment))
    }
}

#[derive(Debug)]
pub struct ExcludeSet {
    indices: BTreeMap<u32, u32>,
}

impl ExcludeSet {
    pub fn new() -> Self {
        ExcludeSet {
            indices: BTreeMap::new(),
        }
    }

    pub fn insert(&mut self, range: Range<u32>, length: u32) {
        debug_assert!(length != 0);
        self.indices.insert(range.start, length);
        self.indices.insert(range.end, 0);
    }

    fn get(&self, index: u32) -> Option<u32> {
        self.indices.get(&index).cloned()
    }
}
