use std::path::Path;

use anyhow::{Context, Result};
use druid::{ArcStr, Data};
use protobuf::{
    descriptor::{FileDescriptorSet, MethodDescriptorProto, ServiceDescriptorProto},
    reflect::{FileDescriptor, MessageDescriptor},
    Message,
};

use crate::protobuf::ProtobufRequest;

#[derive(Clone, Debug)]
pub struct ProtobufService {
    name: String,
    methods: Vec<ProtobufMethod>,
}

#[derive(Debug, Clone)]
pub struct ProtobufMethod {
    name: ArcStr,
    request: MessageDescriptor,
    request_streaming: bool,
    response: MessageDescriptor,
    response_streaming: bool,
}

impl ProtobufService {
    pub fn load(path: &Path) -> Result<Vec<Self>> {
        let mut file = fs_err::File::open(path)?;

        let descriptor_set = FileDescriptorSet::parse_from_reader(&mut file)?;

        let files = FileDescriptor::new_dynamic_fds(descriptor_set.file);

        files
            .iter()
            .flat_map(|file| &file.proto().service)
            .map(|service| ProtobufService::new(service, &files))
            .collect()
    }

    fn new(proto: &ServiceDescriptorProto, files: &[FileDescriptor]) -> Result<Self> {
        Ok(ProtobufService {
            name: proto.get_name().to_owned(),
            methods: proto
                .method
                .iter()
                .map(|method| ProtobufMethod::new(method, files))
                .collect::<Result<Vec<_>>>()?,
        })
    }

    pub fn name(&self) -> &str {
        &self.name
    }

    pub fn methods<'a>(&'a self) -> impl Iterator<Item = ProtobufMethod> + 'a {
        self.methods.iter().cloned()
    }
}

impl ProtobufMethod {
    fn new(proto: &MethodDescriptorProto, files: &[FileDescriptor]) -> Result<Self> {
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

        Ok(ProtobufMethod {
            name: proto.get_name().into(),
            request: find_type(proto.get_input_type(), files)?,
            request_streaming: proto.has_client_streaming(),
            response: find_type(proto.get_output_type(), files)?,
            response_streaming: proto.has_server_streaming(),
        })
    }

    pub fn name(&self) -> &ArcStr {
        &self.name
    }

    pub fn request(&self) -> ProtobufRequest {
        ProtobufRequest::new(self.request.clone())
    }
}

impl Data for ProtobufMethod {
    fn same(&self, other: &Self) -> bool {
        self.name.same(&other.name)
            && self.request == other.request
            && self.response == other.response
    }
}
