use std::{
    sync::Arc,
    time::{Duration, Instant},
};

use anyhow::Result;
use futures::{StreamExt, TryStreamExt};
use http::Uri;
use protobuf::MessageDyn;
use tokio::sync::mpsc;
use tokio_stream::wrappers::UnboundedReceiverStream;
use tonic::{client::Grpc, transport::Channel, IntoRequest};

use crate::protobuf::{ProtobufMethod, ProtobufMethodKind};

pub type ConnectResult = Result<Client, (Uri, Error)>;
pub type ResponseResult = Result<Response, Error>;

#[derive(Debug, Clone)]
pub struct Request {
    pub body: Box<dyn MessageDyn>,
}

#[derive(Debug, Clone)]
pub struct Response {
    pub body: Box<dyn MessageDyn>,
    pub timestamp: Instant,
}

#[derive(Debug)]
pub struct Call {
    last_request: Option<Instant>,
    request_sender: Option<mpsc::UnboundedSender<Request>>,
}

pub type Error = Arc<anyhow::Error>;

#[derive(Clone, Debug)]
pub struct Client {
    uri: Uri,
    grpc: Grpc<Channel>,
}

impl Client {
    pub async fn new(uri: Uri) -> ConnectResult {
        let channel = match Channel::builder(uri.clone()).connect().await {
            Ok(channel) => channel,
            Err(err) => return Err((uri, arc_err(err))),
        };
        Ok(Client {
            uri,
            grpc: Grpc::new(channel),
        })
    }

    pub fn uri(&self) -> &Uri {
        &self.uri
    }

    pub fn call<F>(mut self, method: &ProtobufMethod, request: Request, mut on_response: F) -> Call
    where
        F: FnMut(Option<ResponseResult>) + Send + 'static,
    {
        let path = method.path();
        let codec = method.codec();

        let last_request = Some(Instant::now());

        let request_sender = match method.kind() {
            ProtobufMethodKind::Unary => {
                tokio::spawn(async move {
                    match self.grpc.unary(request.into_request(), path, codec).await {
                        Ok(response) => on_response(Some(Ok(response.into_inner()))),
                        Err(err) => on_response(Some(Err(arc_err(err)))),
                    }
                    on_response(None);
                });

                None
            }
            ProtobufMethodKind::ClientStreaming => {
                let (request_sender, request_receiver) = mpsc::unbounded_channel();

                request_sender.send(request).unwrap();

                tokio::spawn(async move {
                    match self
                        .grpc
                        .client_streaming(
                            UnboundedReceiverStream::new(request_receiver).into_request(),
                            path,
                            codec,
                        )
                        .await
                    {
                        Ok(response) => on_response(Some(Ok(response.into_inner()))),
                        Err(err) => on_response(Some(Err(arc_err(err)))),
                    }
                    on_response(None);
                });

                Some(request_sender)
            }
            ProtobufMethodKind::ServerStreaming => {
                tokio::spawn(async move {
                    match self
                        .grpc
                        .server_streaming(request.into_request(), path, codec)
                        .await
                    {
                        Ok(stream) => {
                            let mut stream = stream.into_inner().map_err(arc_err);
                            while let Some(result) = stream.next().await {
                                let is_err = result.is_err();
                                on_response(Some(result));
                                if is_err {
                                    break;
                                }
                            }
                        }
                        Err(err) => {
                            on_response(Some(Err(arc_err(err))));
                        }
                    }

                    on_response(None);
                });

                None
            }
            ProtobufMethodKind::Streaming => {
                let (request_sender, request_receiver) = mpsc::unbounded_channel();

                request_sender.send(request).unwrap();

                tokio::spawn(async move {
                    match self
                        .grpc
                        .streaming(
                            UnboundedReceiverStream::new(request_receiver).into_request(),
                            path,
                            codec,
                        )
                        .await
                    {
                        Ok(stream) => {
                            let mut stream = stream.into_inner().map_err(arc_err);
                            while let Some(result) = stream.next().await {
                                let is_err = result.is_err();
                                on_response(Some(result));
                                if is_err {
                                    break;
                                }
                            }
                        }
                        Err(err) => {
                            on_response(Some(Err(arc_err(err))));
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
}

impl Request {
    pub fn body(&self) -> &dyn MessageDyn {
        &*self.body
    }

    pub fn body_mut(&mut self) -> &mut dyn MessageDyn {
        &mut *self.body
    }
}

impl Response {
    pub fn new(body: Box<dyn MessageDyn>) -> Self {
        Response {
            body,
            timestamp: Instant::now(),
        }
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

fn arc_err(err: impl Into<anyhow::Error>) -> Error {
    Arc::new(err.into())
}
