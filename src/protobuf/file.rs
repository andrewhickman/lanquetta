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

use crate::{
    data_dir,
    protobuf::{ProtobufCodec, ProtobufMessage},
    sha,
};

#[derive(Clone, Debug)]
pub struct ProtobufService {
    name: ArcStr,
    methods: Vec<ProtobufMethod>,
    path: Arc<Path>,
}

#[derive(Debug, Clone)]
pub struct ProtobufMethod {
    name: ArcStr,
    service_name: ArcStr,
    request: MessageDescriptor,
    request_streaming: bool,
    response: MessageDescriptor,
    response_streaming: bool,
}

impl ProtobufService {
    pub fn load(path: &Path) -> Result<Vec<Self>> {
        let (bytes, path) = load_persisted_file(path)?;

        let descriptor_set = FileDescriptorSet::parse_from_bytes(&bytes)?;

        let files = FileDescriptor::new_dynamic_fds(descriptor_set.file);

        files
            .iter()
            .flat_map(|file| &file.proto().service)
            .map(|service| ProtobufService::new(service, &files, path.clone()))
            .collect()
    }

    fn new(
        proto: &ServiceDescriptorProto,
        files: &[FileDescriptor],
        path: Arc<Path>,
    ) -> Result<Self> {
        let name: ArcStr = proto.get_name().into();
        Ok(ProtobufService {
            methods: proto
                .method
                .iter()
                .map(|method| ProtobufMethod::new(name.clone(), method, files))
                .collect::<Result<Vec<_>>>()?,
            name,
            path,
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
    fn new(
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

        Ok(ProtobufMethod {
            service_name,
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
}

impl Data for ProtobufMethod {
    fn same(&self, other: &Self) -> bool {
        self.name.same(&other.name)
            && self.request == other.request
            && self.response == other.response
    }
}

fn load_persisted_file(path: &Path) -> Result<(Vec<u8>, Arc<Path>)> {
    let bytes = fs_err::read(path)?;
    match data_dir() {
        Some(data_dir) => {
            fs_err::create_dir_all(&data_dir)?;
            let hash = sha::hash(&bytes);
            let persisted_path = data_dir.join(hash);

            if !persisted_path.exists() {
                log::debug!("Persisting file to {}", persisted_path.display());
                fs_err::write(&persisted_path, &bytes)?;
            }
            Ok((bytes, persisted_path.into()))
        }
        None => {
            log::warn!("Data directory not found");
            Ok((bytes, path.into()))
        }
    }
}
