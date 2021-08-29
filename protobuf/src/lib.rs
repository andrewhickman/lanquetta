mod serde;
mod ty;

use std::{path::Path, str::FromStr, sync::Arc};

use anyhow::Result;
use druid::{ArcStr, Data};
use http::uri::PathAndQuery;
use prost::bytes::Buf;
use prost_types::{FileDescriptorProto, FileDescriptorSet, MethodDescriptorProto, ServiceDescriptorProto};

#[derive(Debug, Clone, Data)]
pub struct FileSet {
    inner: Arc<FileSetInner>,
}

#[derive(Debug)]
struct FileSetInner {
    raw: FileDescriptorSet,
    type_map: ty::TypeMap,
    services: Vec<ServiceInner>,
}

#[derive(Debug, Clone, Data)]
pub struct Service {
    file_set: FileSet,
    index: usize,
}

#[derive(Debug)]
struct ServiceInner {
    name: ArcStr,
    methods: Vec<MethodInner>,
}

#[derive(Debug, Clone, Data)]
pub struct Method {
    service: Service,
    index: usize,
}

#[derive(Debug)]
struct MethodInner {
    name: ArcStr,
    kind: MethodKind,
    path: PathAndQuery,
    request_ty: ty::TypeId,
    response_ty: ty::TypeId,
}

#[derive(Debug, Copy, Clone, Data, Eq, PartialEq)]
pub enum MethodKind {
    Unary = 0b00,
    ClientStreaming = 0b01,
    ServerStreaming = 0b10,
    Streaming = 0b11,
}

#[derive(Debug, Clone, Data)]
pub struct Message {
    file_set: FileSet,
    ty: ty::TypeId,
}

#[derive(Debug)]
struct MessageInner {
    file_set: FileSet,
    index: usize,
}

impl FileSet {
    pub fn from_file<P>(path: P) -> Result<Self>
    where
        P: AsRef<Path>,
    {
        FileSet::from_bytes(fs_err::read(path)?.as_ref())
    }

    pub fn from_bytes<B>(bytes: B) -> Result<Self>
    where
        B: Buf,
    {
        Ok(FileSet {
            inner: Arc::new(FileSet::from_raw(prost::Message::decode(bytes)?)?),
        })
    }

    fn from_raw(raw: FileDescriptorSet) -> Result<FileSetInner> {
        let type_map = ty::TypeMap::new(&raw)?;
        let type_map_ref = &type_map;

        let services = raw
            .file
            .iter()
            .flat_map(|raw_file| raw_file.service
                .iter()
                .map(move |raw_service| Service::from_raw(raw_file, raw_service, type_map_ref))
            )
            .collect::<Result<_>>()?;

        Ok(FileSetInner {
            raw,
            type_map,
            services,
        })
    }

    pub fn to_bytes(&self) -> Vec<u8> {
        prost::Message::encode_to_vec(&self.inner.raw)
    }

    pub fn services(&self) -> impl ExactSizeIterator<Item = Service> + '_ {
        (0..self.inner.services.len()).map(move |index| Service {
            file_set: self.clone(),
            index,
        })
    }

    pub fn get_service(&self, index: usize) -> Option<Service> {
        if index < self.inner.services.len() {
            Some(Service {
                file_set: self.clone(),
                index,
            })
        } else {
            None
        }
    }
}

impl Service {
    fn from_raw(raw_file: &FileDescriptorProto, raw_service: &ServiceDescriptorProto, type_map: &ty::TypeMap) -> Result<ServiceInner> {
        let methods = raw_service
            .method
            .iter()
            .map(|raw_method| Method::from_raw(raw_file, raw_service, raw_method, type_map))
            .collect::<Result<_>>()?;
        Ok(ServiceInner {
            name: raw_service.name().into(),
            methods,
        })
    }

    pub fn file_set(&self) -> &FileSet {
        &self.file_set
    }

    pub fn index(&self) -> usize {
        self.index
    }

    pub fn name(&self) -> ArcStr {
        self.inner().name.clone()
    }

    pub fn methods(&self) -> impl ExactSizeIterator<Item = Method> + '_ {
        (0..self.inner().methods.len()).map(move |index| Method {
            service: self.clone(),
            index,
        })
    }

    pub fn get_method(&self, index: usize) -> Option<Method> {
        if index < self.inner().methods.len() {
            Some(Method {
                service: self.clone(),
                index,
            })
        } else {
            None
        }
    }

    fn inner(&self) -> &ServiceInner {
        &self.file_set().inner.services[self.index]
    }
}

impl Method {
    fn from_raw(raw_file: &FileDescriptorProto, raw_service: &ServiceDescriptorProto, raw_method: &MethodDescriptorProto, type_map: &ty::TypeMap) -> Result<MethodInner> {
        let kind = match (raw_method.client_streaming(), raw_method.server_streaming()) {
            (false, false) => MethodKind::Unary,
            (true, false) => MethodKind::ClientStreaming,
            (false, true) => MethodKind::ServerStreaming,
            (true, true) => MethodKind::Streaming,
        };

        let namespace = match raw_file.package() {
            "" => String::default(),
            package => format!(".{}", package),
        };
        let path =  PathAndQuery::from_str(&format!(
            "/{}{}/{}",
            namespace,
            raw_service.name(),
            raw_method.name()
        ))?;

        let request_ty = type_map.get_by_name(raw_method.input_type())?;
        let response_ty = type_map.get_by_name(raw_method.output_type())?;

        Ok(MethodInner {
            name: raw_method.name().into(),
            kind,
            path,
            request_ty,
            response_ty,
        })
    }

    pub fn file_set(&self) -> &FileSet {
        self.service.file_set()
    }

    pub fn index(&self) -> usize {
        self.index
    }

    pub fn name(&self) -> ArcStr {
        self.inner().name.clone()
    }

    pub fn kind(&self) -> MethodKind {
        self.inner().kind
    }

    pub fn path(&self) -> PathAndQuery {
        self.inner().path.clone()
    }

    pub fn request(&self) -> Message {
        Message {
            file_set: self.file_set().clone(),
            ty: self.inner().request_ty,
        }
    }

    pub fn response(&self) -> Message {
        Message {
            file_set: self.file_set().clone(),
            ty: self.inner().response_ty,
        }
    }

    fn inner(&self) -> &MethodInner {
        &self.service.inner().methods[self.index]
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
