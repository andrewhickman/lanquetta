use std::cmp::Ordering;
use std::collections::BinaryHeap;
use std::io::{self, Sink};
use std::ops::Range;

use anyhow::Result;
use serde_json::ser::Formatter;

use super::exclude::ExcludeSet;
use super::ShortenOptions;

pub fn count<F>(opts: ShortenOptions, f: F) -> Result<ExcludeSet>
where
    F: FnOnce(&mut serde_json::Serializer<Sink, &mut Counter>) -> Result<()>,
{
    let mut counter = Counter {
        opts,
        position: 0,
        depth: 0,
        length: 1,
        stack: Vec::new(),
        objects: BinaryHeap::new(),
    };

    let mut ser = serde_json::Serializer::with_formatter(io::sink(), &mut counter);
    f(&mut ser)?;

    let mut excludes = ExcludeSet::new();
    if let Some(max_length) = opts.max_length {
        while counter.length > max_length {
            let max = counter.objects.pop().unwrap();
            if max.length != 0 {
                excludes.insert(max.range, max.length - 1);
                counter.length -= max.length;
            }
        }
    }

    if let Some(max_depth) = opts.max_depth {
        while let Some(max) = counter.objects.pop() {
            debug_assert!(max.depth <= max_depth);
            if max.depth == max_depth && max.length != 0 {
                excludes.insert(max.range, max.length - 1);
                counter.length -= max.length;
            }
        }
    }

    Ok(excludes)
}

pub struct Counter {
    opts: ShortenOptions,
    position: u32,
    depth: u32,
    length: u32,
    stack: Vec<HalfObject>,
    objects: BinaryHeap<Object>,
}

impl Counter {
    fn begin(&mut self) {
        if !self.skip() {
            self.stack.push(HalfObject {
                length: 0,
                start: self.position,
            });
        }
        self.depth += 1;
        self.position += 1;
    }

    fn end(&mut self) {
        self.depth -= 1;
        if !self.skip() {
            let HalfObject { start, mut length } = self.stack.pop().unwrap();
            if length != 0 {
                length += 1;
                self.length += 1;
            }
            self.objects.push(Object {
                depth: self.depth,
                length,
                range: start..self.position,
            });
        }
        self.position += 1;
    }

    fn skip(&self) -> bool {
        match self.opts.max_depth {
            Some(max) => self.depth > max,
            None => false,
        }
    }

    fn skip1(&self) -> bool {
        match self.opts.max_depth {
            Some(max) => self.depth > max + 1,
            None => false,
        }
    }

    fn incr(&mut self) {
        if !self.skip1() {
            self.stack.last_mut().unwrap().length += 1;
            self.length += 1;
        }
    }
}

impl Formatter for &'_ mut Counter {
    fn begin_array<W: ?Sized>(&mut self, _: &mut W) -> io::Result<()> {
        self.begin();
        Ok(())
    }

    fn end_array<W: ?Sized>(&mut self, _: &mut W) -> io::Result<()> {
        self.end();
        Ok(())
    }

    fn begin_array_value<W: ?Sized>(&mut self, _: &mut W, _: bool) -> io::Result<()> {
        self.incr();
        Ok(())
    }

    fn end_array_value<W: ?Sized>(&mut self, _: &mut W) -> io::Result<()> {
        Ok(())
    }

    fn begin_object<W: ?Sized>(&mut self, _: &mut W) -> io::Result<()> {
        self.begin();
        Ok(())
    }

    fn end_object<W: ?Sized>(&mut self, _: &mut W) -> io::Result<()> {
        self.end();
        Ok(())
    }

    fn begin_object_key<W: ?Sized>(&mut self, _: &mut W, _: bool) -> io::Result<()> {
        self.incr();
        Ok(())
    }

    fn begin_object_value<W: ?Sized>(&mut self, _: &mut W) -> io::Result<()> {
        Ok(())
    }

    fn end_object_value<W: ?Sized>(&mut self, _: &mut W) -> io::Result<()> {
        Ok(())
    }
}

#[derive(Copy, Clone, Debug)]
struct HalfObject {
    start: u32,
    length: u32,
}

#[derive(Clone, Debug)]
struct Object {
    // The depth this object is at.
    depth: u32,
    // The number of lines this object takes up.
    length: u32,
    // The position of this object in the iteration order.
    range: Range<u32>,
}

impl PartialEq for Object {
    fn eq(&self, other: &Self) -> bool {
        self.cmp(other) == Ordering::Equal
    }
}

impl Eq for Object {}

impl PartialOrd for Object {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for Object {
    fn cmp(&self, other: &Self) -> Ordering {
        self.depth.cmp(&other.depth).then(
            self.length
                .cmp(&other.length)
                .then(self.range.start.cmp(&other.range.start)),
        )
    }
}
