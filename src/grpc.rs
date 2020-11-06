use std::{sync::Arc, time::Duration};

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

    pub async fn send(self, request: Request) -> ResponseResult {
        #![allow(unused)]

        tokio::time::delay_for(Duration::from_secs(2)).await;
        return Ok(Response {
            body: request.method.response().empty(),
        });

        let body = self
            .grpc
            .unary(
                request.body.into_request(),
                request.method.path()?,
                request.method.codec(),
            )
            .await
            .map_err(arc_err)?;

        Ok(Response {
            body: body.into_inner(),
        })
    }
}

fn arc_err(err: impl Into<anyhow::Error>) -> Error {
    Arc::new(err.into())
}
