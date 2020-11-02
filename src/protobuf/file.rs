use std::path::Path;
use std::str::FromStr;
use std::sync::Arc;

use anyhow::{Context, Result};
use druid::{ArcStr, Data};
use http::uri::PathAndQuery;
use protobuf::{
    descriptor::{FileDescriptorSet, MethodDescriptorProto, ServiceDescriptorProto},
    reflect::{FileDescriptor, MessageDescriptor},
    Message,
};

use crate::protobuf::{ProtobufCodec, ProtobufMessage, ProtobufMethodKind};

#[derive(Clone, Debug)]
pub struct ProtobufService {
    name: ArcStr,
    methods: Vec<ProtobufMethod>,
    fd_set: Arc<FileDescriptorSet>,
    service_index: usize,
}

#[derive(Debug, Clone)]
pub struct ProtobufMethod {
    fd_set: Arc<FileDescriptorSet>,
    service_index: usize,
    method_index: usize,
    name: ArcStr,
    service_name: ArcStr,
    kind: ProtobufMethodKind,
    request: MessageDescriptor,
    response: MessageDescriptor,
}

impl ProtobufService {
    pub fn load(descriptor_set: &Arc<FileDescriptorSet>) -> Result<Vec<Self>> {
        let files = FileDescriptor::new_dynamic_fds(descriptor_set.file.clone());

        files
            .iter()
            .flat_map(|file| &file.proto().service)
            .enumerate()
            .map(|(index, service)| {
                ProtobufService::new(descriptor_set.clone(), index, service, &files)
            })
            .collect()
    }

    pub fn load_file(path: &Path) -> Result<Vec<Self>> {
        let mut file = fs_err::File::open(path)?;

        let descriptor_set = Arc::new(FileDescriptorSet::parse_from_reader(&mut file)?);

        ProtobufService::load(&descriptor_set)
    }

    fn new(
        fd_set: Arc<FileDescriptorSet>,
        service_index: usize,
        proto: &ServiceDescriptorProto,
        files: &[FileDescriptor],
    ) -> Result<Self> {
        let name: ArcStr = proto.get_name().into();
        Ok(ProtobufService {
            methods: proto
                .method
                .iter()
                .enumerate()
                .map(|(method_index, method)| {
                    ProtobufMethod::new(
                        fd_set.clone(),
                        service_index,
                        method_index,
                        name.clone(),
                        method,
                        files,
                    )
                })
                .collect::<Result<Vec<_>>>()?,
            fd_set,
            service_index,
            name,
        })
    }

    pub fn name(&self) -> &str {
        &self.name
    }

    pub fn get_method(&self, index: usize) -> Option<&ProtobufMethod> {
        self.methods.get(index)
    }

    pub fn methods<'a>(&'a self) -> impl Iterator<Item = ProtobufMethod> + 'a {
        self.methods.iter().cloned()
    }

    pub fn fd_set(&self) -> Arc<FileDescriptorSet> {
        self.fd_set.clone()
    }

    pub fn service_index(&self) -> usize {
        self.service_index
    }
}

impl ProtobufMethod {
    fn new(
        fd_set: Arc<FileDescriptorSet>,
        service_index: usize,
        method_index: usize,
        service_name: ArcStr,
        proto: &MethodDescriptorProto,
        files: &[FileDescriptor],
    ) -> Result<Self> {
        fn find_type(full_name: &str, files: &[FileDescriptor]) -> Result<MessageDescriptor> {
            files
                .iter()
                .find_map(|file| file.message_by_full_name(full_name))
                .with_context(|| {
                    format!(
                        "invalid file descriptor set: type '{}' not found",
                        full_name
                    )
                })
        }

        let kind = match (proto.has_client_streaming(), proto.has_server_streaming()) {
            (false, false) => ProtobufMethodKind::Unary,
            (true, false) => ProtobufMethodKind::ClientStreaming,
            (false, true) => ProtobufMethodKind::ServerStreaming,
            (true, true) => ProtobufMethodKind::Streaming,
        };

        Ok(ProtobufMethod {
            fd_set,
            service_index,
            method_index,
            service_name,
            name: proto.get_name().into(),
            kind,
            request: find_type(proto.get_input_type(), files)?,
            response: find_type(proto.get_output_type(), files)?,
        })
    }

    pub fn name(&self) -> &ArcStr {
        &self.name
    }

    pub fn kind(&self) -> ProtobufMethodKind {
        self.kind
    }

    pub fn request(&self) -> ProtobufMessage {
        ProtobufMessage::new(self.request.clone())
    }

    pub fn response(&self) -> ProtobufMessage {
        ProtobufMessage::new(self.response.clone())
    }

    pub fn codec(&self) -> ProtobufCodec {
        ProtobufCodec::new(self.request.clone(), self.response.clone())
    }

    pub fn path(&self) -> Result<PathAndQuery> {
        Ok(PathAndQuery::from_str(&format!(
            "{}/{}",
            self.service_name, self.name
        ))?)
    }

    pub fn fd_set(&self) -> Arc<FileDescriptorSet> {
        self.fd_set.clone()
    }

    pub fn service_index(&self) -> usize {
        self.service_index
    }

    pub fn method_index(&self) -> usize {
        self.method_index
    }
}

impl Data for ProtobufMethod {
    fn same(&self, other: &Self) -> bool {
        self.name.same(&other.name)
            && self.request == other.request
            && self.response == other.response
    }
}
