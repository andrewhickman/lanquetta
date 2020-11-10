use std::sync::Arc;

use anyhow::Result;
use http::Uri;
use protobuf::MessageDyn;
use tonic::{client::Grpc, transport::Channel, IntoRequest};

use crate::protobuf::ProtobufMethod;

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

    pub async fn send(mut self, request: Request) -> ResponseResult {
        let path = request.method.path();
        let codec = request.method.codec();

        let response = self
            .grpc
            .unary(
                request.into_request(),
                path,
                codec,
            )
            .await
            .map_err(arc_err)?;

        Ok(response.into_inner())
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
