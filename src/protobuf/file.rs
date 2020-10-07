use std::path::Path;
use std::sync::Arc;

use anyhow::{Context, Result};
use druid::widget::ListIter;
use druid::Data;
use protobuf::descriptor::{FileDescriptorSet, MethodDescriptorProto, ServiceDescriptorProto};
use protobuf::reflect::{FileDescriptor, MessageDescriptor};
use protobuf::Message;

#[derive(Clone, Debug, Data)]
pub struct ProtobufService {
    methods: Arc<Vec<ProtobufMethod>>,
}

#[derive(Debug, Clone)]
pub struct ProtobufMethod {
    name: String,
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
            methods: Arc::new(
                proto
                    .method
                    .iter()
                    .map(|method| ProtobufMethod::new(method, files))
                    .collect::<Result<Vec<_>>>()?,
            ),
        })
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
            name: proto.get_name().to_owned(),
            request: find_type(proto.get_input_type(), files)?,
            request_streaming: proto.has_client_streaming(),
            response: find_type(proto.get_output_type(), files)?,
            response_streaming: proto.has_server_streaming(),
        })
    }
}

impl Data for ProtobufMethod {
    fn same(&self, other: &Self) -> bool {
        self.name.same(&other.name)
            && self.request == other.request
            && self.response == other.response
    }
}

impl ListIter<ProtobufMethod> for ProtobufService {
    fn for_each(&self, cb: impl FnMut(&ProtobufMethod, usize)) {
        self.methods.for_each(cb)
    }

    fn for_each_mut(&mut self, cb: impl FnMut(&mut ProtobufMethod, usize)) {
        self.methods.for_each_mut(cb)
    }

    fn data_len(&self) -> usize {
        self.methods.data_len()
    }
}
