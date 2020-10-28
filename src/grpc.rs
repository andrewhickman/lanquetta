use std::time::Duration;

use futures::future::FutureExt;
use http::Uri;
use protobuf::MessageDyn;
use tonic::{client::Grpc, transport::Channel, IntoRequest};

use crate::protobuf::ProtobufMethod;

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

pub type Error = anyhow::Error;

#[derive(Clone)]
pub struct Client {
    grpc: Grpc<Channel>,
}

impl Client {
    pub fn new(uri: Uri) -> Result<Self, tonic::transport::Error> {
        Ok(Client {
            grpc: Grpc::new(Channel::builder(uri).connect_lazy()?),
        })
    }

    pub fn send(&self, request: Request, callback: impl FnOnce(ResponseResult) + Send + 'static) {
        tokio::spawn(self.clone().send_impl(request).map(callback));
    }

    async fn send_impl(self, request: Request) -> ResponseResult {
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
            .await?;

        Ok(Response {
            body: body.into_inner(),
        })
    }
}
