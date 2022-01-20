mod channel;
mod codec;

use std::{
    str::FromStr,
    sync::Arc,
    time::{Duration, Instant},
};

use anyhow::Result;
use futures::{Stream, StreamExt, TryStreamExt};
use http::{uri::PathAndQuery, Uri};
use prost_reflect::{DeserializeOptions, DynamicMessage, MessageDescriptor, SerializeOptions};
use tokio::sync::mpsc;
use tokio_stream::wrappers::UnboundedReceiverStream;
use tonic::{client::Grpc, transport::Channel, IntoRequest};

pub type ConnectResult = Result<Client, Error>;
pub type ResponseResult = Result<Response, Error>;

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

pub type Error = Arc<anyhow::Error>;

#[derive(Clone, Debug)]
pub struct Client {
    grpc: Grpc<Channel>,
}

impl Client {
    pub async fn new(uri: &Uri) -> ConnectResult {
        let channel = channel::get(uri).await?;
        Ok(Client {
            grpc: Grpc::new(channel),
        })
    }

    pub fn call<F>(
        self,
        method: prost_reflect::MethodDescriptor,
        request: Request,
        mut on_response: F,
    ) -> Call
    where
        F: FnMut(Option<ResponseResult>) + Send + 'static,
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
                    match self.unary(&method, request, path).await {
                        Ok(response) => on_response(Some(Ok(response))),
                        Err(err) => on_response(Some(Err(err.into()))),
                    }
                    on_response(None);
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
                            path,
                        )
                        .await
                    {
                        Ok(response) => on_response(Some(Ok(response))),
                        Err(err) => on_response(Some(Err(err.into()))),
                    }
                    on_response(None);
                });

                Some(request_sender)
            }
            MethodKind::ServerStreaming => {
                tokio::spawn(async move {
                    match self.server_streaming(&method, request, path).await {
                        Ok(stream) => {
                            let mut stream = stream.map_err(arc_err);
                            while let Some(result) = stream.next().await {
                                let is_err = result.is_err();
                                on_response(Some(result));
                                if is_err {
                                    break;
                                }
                            }
                        }
                        Err(err) => {
                            on_response(Some(Err(err.into())));
                        }
                    }

                    on_response(None);
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
                            path,
                        )
                        .await
                    {
                        Ok(stream) => {
                            let mut stream = stream.map_err(arc_err);
                            while let Some(result) = stream.next().await {
                                let is_err = result.is_err();
                                on_response(Some(result));
                                if is_err {
                                    break;
                                }
                            }
                        }
                        Err(err) => {
                            on_response(Some(Err(err.into())));
                        }
                    }

                    on_response(None);
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
        path: PathAndQuery,
    ) -> anyhow::Result<Response> {
        self.grpc.ready().await?;
        Ok(self
            .grpc
            .unary(
                request.into_request(),
                path,
                codec::DynamicCodec::new(method.clone()),
            )
            .await?
            .into_inner())
    }

    async fn client_streaming(
        mut self,
        method: &prost_reflect::MethodDescriptor,
        requests: impl Stream<Item = Request> + Send + Sync + 'static,
        path: PathAndQuery,
    ) -> anyhow::Result<Response> {
        self.grpc.ready().await?;
        Ok(self
            .grpc
            .client_streaming(
                requests.into_request(),
                path,
                codec::DynamicCodec::new(method.clone()),
            )
            .await?
            .into_inner())
    }

    async fn server_streaming(
        mut self,
        method: &prost_reflect::MethodDescriptor,
        request: Request,
        path: PathAndQuery,
    ) -> anyhow::Result<tonic::Streaming<Response>> {
        self.grpc.ready().await?;
        Ok(self
            .grpc
            .server_streaming(
                request.into_request(),
                path,
                codec::DynamicCodec::new(method.clone()),
            )
            .await?
            .into_inner())
    }

    async fn streaming(
        mut self,
        method: &prost_reflect::MethodDescriptor,
        requests: impl Stream<Item = Request> + Send + Sync + 'static,
        path: PathAndQuery,
    ) -> anyhow::Result<tonic::Streaming<Response>> {
        self.grpc.ready().await?;
        Ok(self
            .grpc
            .streaming(
                requests.into_request(),
                path,
                codec::DynamicCodec::new(method.clone()),
            )
            .await?
            .into_inner())
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

    pub fn duration(&mut self, response: &Response) -> Option<Duration> {
        self.last_request.take().and_then(|request_timestamp| {
            response.timestamp.checked_duration_since(request_timestamp)
        })
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

fn arc_err(err: impl Into<anyhow::Error>) -> Error {
    Arc::new(err.into())
}
