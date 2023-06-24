mod channel;
mod codec;

use std::{
    str::FromStr,
    time::{Duration, Instant},
};

use anyhow::{Error, Result};
use futures::{Stream, StreamExt};
use http::{uri::PathAndQuery, HeaderMap, Uri};
use prost_reflect::{DeserializeOptions, DynamicMessage, MessageDescriptor, SerializeOptions};
use tokio::sync::mpsc;
use tokio_stream::wrappers::UnboundedReceiverStream;
use tonic::{client::Grpc, metadata::MetadataMap, transport::Channel, Extensions, Status};

pub type ConnectResult = Result<Client>;

pub enum ResponseResult {
    Metadata(MetadataMap),
    Response(Response),
    Finished,
    Error(Error, MetadataMap),
}

#[derive(Debug, Clone)]
pub struct Request {
    pub message: DynamicMessage,
}

#[derive(Debug, Clone)]
pub struct Response {
    pub message: DynamicMessage,
    pub timestamp: Instant,
}

#[derive(Debug, Copy, Clone, druid::Data, PartialEq, Eq)]
pub enum MethodKind {
    Unary,
    ClientStreaming,
    ServerStreaming,
    Streaming,
}

#[derive(Debug)]
pub struct Call {
    last_request: Option<Instant>,
    request_sender: Option<mpsc::UnboundedSender<Request>>,
}

#[derive(Clone, Debug)]
pub struct Client {
    grpc: Grpc<Channel>,
}

impl Client {
    pub async fn new(uri: &Uri, verify_certs: bool) -> ConnectResult {
        let channel = channel::get(uri, verify_certs).await?;
        Ok(Client {
            grpc: Grpc::new(channel),
        })
    }

    pub fn call<F>(
        self,
        method: prost_reflect::MethodDescriptor,
        request: Request,
        metadata: MetadataMap,
        mut on_response: F,
    ) -> Call
    where
        F: FnMut(ResponseResult) + Send + 'static,
    {
        let path = PathAndQuery::from_str(&format!(
            "/{}/{}",
            method.parent_service().full_name(),
            method.name()
        ))
        .unwrap();

        let last_request = Some(Instant::now());

        let request_sender = match MethodKind::for_method(&method) {
            MethodKind::Unary => {
                tokio::spawn(async move {
                    match self.unary(&method, request, metadata, path).await {
                        Ok(response) => {
                            let (metadata, message, _) = response.into_parts();
                            on_response(ResponseResult::Response(message));
                            on_response(ResponseResult::Metadata(metadata));
                            on_response(ResponseResult::Finished);
                        }
                        Err(err) => on_response(ResponseResult::from_status(err)),
                    }
                });

                None
            }
            MethodKind::ClientStreaming => {
                let (request_sender, request_receiver) = mpsc::unbounded_channel();

                request_sender.send(request).unwrap();

                tokio::spawn(async move {
                    match self
                        .client_streaming(
                            &method,
                            UnboundedReceiverStream::new(request_receiver),
                            metadata,
                            path,
                        )
                        .await
                    {
                        Ok(response) => {
                            let (metadata, message, _) = response.into_parts();
                            on_response(ResponseResult::Response(message));
                            on_response(ResponseResult::Metadata(metadata));
                            on_response(ResponseResult::Finished);
                        }
                        Err(err) => on_response(ResponseResult::from_status(err)),
                    }
                });

                Some(request_sender)
            }
            MethodKind::ServerStreaming => {
                tokio::spawn(async move {
                    match self
                        .server_streaming(&method, request, metadata, path)
                        .await
                    {
                        Ok((mut stream, metadata)) => {
                            on_response(ResponseResult::Metadata(metadata));
                            loop {
                                match stream.next().await {
                                    Some(Ok(response)) => {
                                        on_response(ResponseResult::Response(response));
                                    }
                                    Some(Err(err)) => {
                                        on_response(ResponseResult::from_status(err));
                                        break;
                                    }
                                    None => {
                                        match stream.trailers().await {
                                            Ok(metadata) => {
                                                on_response(ResponseResult::Metadata(
                                                    metadata.unwrap_or_default(),
                                                ));
                                                on_response(ResponseResult::Finished);
                                            }
                                            Err(err) => {
                                                on_response(ResponseResult::from_status(err));
                                            }
                                        }
                                        break;
                                    }
                                }
                            }
                        }
                        Err(err) => {
                            on_response(ResponseResult::from_status(err));
                        }
                    }
                });

                None
            }
            MethodKind::Streaming => {
                let (request_sender, request_receiver) = mpsc::unbounded_channel();

                request_sender.send(request).unwrap();

                tokio::spawn(async move {
                    match self
                        .streaming(
                            &method,
                            UnboundedReceiverStream::new(request_receiver),
                            metadata,
                            path,
                        )
                        .await
                    {
                        Ok((mut stream, metadata)) => {
                            on_response(ResponseResult::Metadata(metadata));
                            loop {
                                match stream.next().await {
                                    Some(Ok(response)) => {
                                        on_response(ResponseResult::Response(response));
                                    }
                                    Some(Err(err)) => {
                                        on_response(ResponseResult::from_status(err));
                                        break;
                                    }
                                    None => {
                                        match stream.trailers().await {
                                            Ok(metadata) => {
                                                on_response(ResponseResult::Metadata(
                                                    metadata.unwrap_or_default(),
                                                ));
                                                on_response(ResponseResult::Finished);
                                            }
                                            Err(err) => {
                                                on_response(ResponseResult::from_status(err));
                                            }
                                        }
                                        break;
                                    }
                                }
                            }
                        }
                        Err(err) => {
                            on_response(ResponseResult::from_status(err));
                        }
                    }
                });

                Some(request_sender)
            }
        };

        Call {
            request_sender,
            last_request,
        }
    }

    async fn unary(
        mut self,
        method: &prost_reflect::MethodDescriptor,
        request: Request,
        metadata: MetadataMap,
        path: PathAndQuery,
    ) -> tonic::Result<tonic::Response<Response>> {
        self.grpc
            .ready()
            .await
            .map_err(|err| Status::from_error(err.into()))?;
        self.grpc
            .unary(
                tonic::Request::from_parts(metadata, Extensions::default(), request),
                path,
                codec::DynamicCodec::new(method.clone()),
            )
            .await
    }

    async fn client_streaming(
        mut self,
        method: &prost_reflect::MethodDescriptor,
        requests: impl Stream<Item = Request> + Send + Sync + 'static,
        metadata: MetadataMap,
        path: PathAndQuery,
    ) -> tonic::Result<tonic::Response<Response>> {
        self.grpc
            .ready()
            .await
            .map_err(|err| Status::from_error(err.into()))?;
        self.grpc
            .client_streaming(
                tonic::Request::from_parts(metadata, Extensions::default(), requests),
                path,
                codec::DynamicCodec::new(method.clone()),
            )
            .await
    }

    async fn server_streaming(
        mut self,
        method: &prost_reflect::MethodDescriptor,
        request: Request,
        metadata: MetadataMap,
        path: PathAndQuery,
    ) -> tonic::Result<(tonic::Streaming<Response>, MetadataMap)> {
        self.grpc
            .ready()
            .await
            .map_err(|err| Status::from_error(err.into()))?;
        let (metadata, stream, _) = self
            .grpc
            .server_streaming(
                tonic::Request::from_parts(metadata, Extensions::default(), request),
                path,
                codec::DynamicCodec::new(method.clone()),
            )
            .await?
            .into_parts();
        Ok((stream, metadata))
    }

    async fn streaming(
        mut self,
        method: &prost_reflect::MethodDescriptor,
        requests: impl Stream<Item = Request> + Send + Sync + 'static,
        metadata: MetadataMap,
        path: PathAndQuery,
    ) -> tonic::Result<(tonic::Streaming<Response>, MetadataMap)> {
        self.grpc
            .ready()
            .await
            .map_err(|err| Status::from_error(err.into()))?;
        let (metadata, stream, _) = self
            .grpc
            .streaming(
                tonic::Request::from_parts(metadata, Extensions::default(), requests),
                path,
                codec::DynamicCodec::new(method.clone()),
            )
            .await?
            .into_parts();
        Ok((stream, metadata))
    }
}

impl Request {
    pub fn from_json(desc: MessageDescriptor, s: &str) -> Result<Self> {
        let mut de = serde_json::Deserializer::from_str(s);
        let message =
            DynamicMessage::deserialize_with_options(desc, &mut de, &DeserializeOptions::new())?;
        de.end()?;
        Ok(Request { message })
    }
}

impl Response {
    pub fn new(message: DynamicMessage) -> Self {
        Response {
            message,
            timestamp: Instant::now(),
        }
    }

    pub fn to_json(&self) -> String {
        let mut s = serde_json::Serializer::new(Vec::new());
        self.message
            .serialize_with_options(
                &mut s,
                &SerializeOptions::new()
                    .stringify_64_bit_integers(false)
                    .skip_default_fields(false),
            )
            .unwrap();

        String::from_utf8(s.into_inner()).unwrap()
    }
}

impl Call {
    pub fn send(&mut self, request: Request) {
        self.last_request = Some(Instant::now());
        let _ = self
            .request_sender
            .as_ref()
            .expect("called 'send' on non client streaming call")
            .send(request);
    }

    pub fn finish(&mut self) {
        self.request_sender = None;
    }

    pub fn duration(&mut self, response: &Response) -> Option<Duration> {
        self.last_request.take().and_then(|request_timestamp| {
            response.timestamp.checked_duration_since(request_timestamp)
        })
    }
}

impl ResponseResult {
    fn from_status(err: tonic::Status) -> Self {
        let mut metadata = HeaderMap::new();
        err.add_header(&mut metadata)
            .expect("headers already validated");

        ResponseResult::Error(err.into(), MetadataMap::from_headers(metadata))
    }
}

impl MethodKind {
    pub(crate) fn for_method(method: &prost_reflect::MethodDescriptor) -> MethodKind {
        match (method.is_client_streaming(), method.is_server_streaming()) {
            (false, false) => MethodKind::Unary,
            (true, false) => MethodKind::ClientStreaming,
            (false, true) => MethodKind::ServerStreaming,
            (true, true) => MethodKind::Streaming,
        }
    }
}
