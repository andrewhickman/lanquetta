use std::sync::Arc;

use anyhow::Result;
use http::Uri;
use protobuf::MessageDyn;
use tokio::sync::mpsc;
use tonic::{client::Grpc, transport::Channel, IntoRequest};
use futures::{StreamExt, TryStreamExt};

use crate::protobuf::{ProtobufMethod, ProtobufMethodKind};

pub type ConnectResult = Result<Client, (Uri, Error)>;
pub type ResponseResult = Result<Response, Error>;

#[derive(Debug, Clone)]
pub struct Request {
    pub method: ProtobufMethod,
    pub body: Box<dyn MessageDyn>,
}

#[derive(Debug, Clone)]
pub struct Response {
    pub body: Box<dyn MessageDyn>,
}

#[derive(Debug)]
pub struct Call {
    request_sender: Option<mpsc::Sender<Request>>,
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

    pub fn call<F>(mut self, request: Request, mut on_response: F) -> Call 
    where F: FnMut(ResponseResult) + Send + 'static
    {
        let path = request.method.path();
        let codec = request.method.codec();

        match request.method.kind() {
            ProtobufMethodKind::Unary => {
                tokio::spawn(async move {
                    let result = match self.grpc.unary(request.into_request(), path, codec).await {
                        Ok(response) => Ok(response.into_inner()),
                        Err(err) => Err(arc_err(err)),
                    };
                    on_response(result);
                });

                Call {
                    request_sender: None,
                }
            }
            ProtobufMethodKind::ClientStreaming => {
                let (request_sender, request_receiver) = mpsc::channel(1);

                tokio::spawn(async move {
                    let result = match self
                        .grpc
                        .client_streaming(request_receiver.into_request(), path, codec)
                        .await
                    {
                        Ok(response) => Ok(response.into_inner()),
                        Err(err) => Err(arc_err(err)),
                    };
                    on_response(result);
                });

                Call {
                    request_sender: Some(request_sender),
                }
            }
            ProtobufMethodKind::ServerStreaming => {
                tokio::spawn(async move {
                    let mut stream = match self
                        .grpc
                        .server_streaming(request.into_request(), path, codec)
                        .await
                    {
                        Ok(stream) => stream.into_inner().map_err(arc_err),
                        Err(err) => {
                            on_response(Err(arc_err(err)));
                            return;
                        }
                    };

                    while let Some(result) = stream.next().await {
                        on_response(result);
                    }
                });

                Call {
                    request_sender: None,
                }
            }
            ProtobufMethodKind::Streaming => {
                let (request_sender, request_receiver) = mpsc::channel(1);

                tokio::spawn(async move {
                    let mut stream = match self
                        .grpc
                        .streaming(request_receiver.into_request(), path, codec)
                        .await
                    {
                        Ok(stream) => stream.into_inner().map_err(arc_err),
                        Err(err) => {
                            on_response(Err(arc_err(err)));
                            return;
                        }
                    };

                    while let Some(result) = stream.next().await {
                        on_response(result);
                    }
                });

                Call {
                    request_sender: Some(request_sender),
                }
            }
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
        Response { body }
    }
}

fn arc_err(err: impl Into<anyhow::Error>) -> Error {
    Arc::new(err.into())
}
