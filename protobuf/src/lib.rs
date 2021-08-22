#![allow(dead_code, unused_variables, unreachable_code)]

mod serde;

use std::path::Path;

use anyhow::Result;
use druid::{ArcStr, Data};
use http::uri::PathAndQuery;

#[derive(Debug, Clone, Data)]
pub struct FileSet {}

#[derive(Debug, Clone, Data)]
pub struct Service {}

#[derive(Debug, Clone, Data)]
pub struct Method {}

#[derive(Debug, Copy, Clone, Data, Eq, PartialEq)]
pub enum MethodKind {
    Unary = 0b00,
    ClientStreaming = 0b01,
    ServerStreaming = 0b10,
    Streaming = 0b11,
}

#[derive(Debug, Clone, Data)]
pub struct Message {}

impl FileSet {
    pub fn from_file<P>(path: P) -> Result<Self>
    where
        P: AsRef<Path>,
    {
        FileSet::from_bytes(fs_err::read(path)?)
    }

    pub fn from_bytes<B>(bytes: B) -> Result<Self>
    where
        B: AsRef<[u8]>,
    {
        todo!()
    }

    pub fn to_bytes(&self) -> Vec<u8> {
        todo!()
    }

    pub fn services(&self) -> impl ExactSizeIterator<Item = Service> {
        todo!();
        std::iter::empty()
    }

    pub fn get_service(&self, index: usize) -> Option<Service> {
        todo!()
    }
}

impl Service {
    pub fn file_set(&self) -> &FileSet {
        todo!()
    }

    pub fn index(&self) -> usize {
        todo!()
    }

    pub fn name(&self) -> ArcStr {
        todo!()
    }

    pub fn methods(&self) -> impl ExactSizeIterator<Item = Method> {
        todo!();
        std::iter::empty()
    }

    pub fn get_method(&self, index: usize) -> Option<Method> {
        todo!()
    }
}

impl Method {
    pub fn file_set(&self) -> &FileSet {
        todo!()
    }

    pub fn index(&self) -> usize {
        todo!()
    }

    pub fn name(&self) -> ArcStr {
        todo!()
    }

    pub fn kind(&self) -> MethodKind {
        todo!()
    }

    pub fn path(&self) -> PathAndQuery {
        todo!()
    }

    pub fn request(&self) -> Message {
        todo!()
    }

    pub fn response(&self) -> Message {
        todo!()
    }
}

impl MethodKind {
    pub fn server_streaming(&self) -> bool {
        match *self {
            MethodKind::Unary | MethodKind::ClientStreaming => false,
            MethodKind::ServerStreaming | MethodKind::Streaming => true,
        }
    }

    pub fn client_streaming(&self) -> bool {
        match *self {
            MethodKind::Unary | MethodKind::ServerStreaming => false,
            MethodKind::ClientStreaming | MethodKind::Streaming => true,
        }
    }
}

impl Message {
    pub fn template_json(&self) -> String {
        todo!()
    }

    pub fn decode(&self, protobuf: &[u8]) -> Result<String> {
        todo!()
    }

    pub fn encode(&self, json: &str) -> Result<Vec<u8>> {
        todo!()
    }
}
