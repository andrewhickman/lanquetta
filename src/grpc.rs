use std::sync::Arc;

use futures::future::FutureExt;
use protobuf::MessageDyn;
use tokio::sync::Mutex;
use tonic::client::Grpc;
use tonic::transport::Channel;
use tonic::IntoRequest;

use crate::protobuf::ProtobufCodec;

pub type ResponseResult = Result<Response, Error>;

#[derive(Debug, Clone, druid::Data)]
pub struct Request {
    pub body: Arc<dyn MessageDyn>,
}

#[derive(Debug, Clone, druid::Data)]
pub struct Response {
    pub body: Arc<dyn MessageDyn>,
}

pub type Error = anyhow::Error;

#[derive(Clone, Default)]
pub struct Client {
    grpc: Option<Grpc<Channel>>,
}

impl Client {
    pub fn new() -> Self {
        Client::default()
    }

    pub fn send(&self, request: Request, callback: impl FnOnce(ResponseResult) + Send + 'static) {
        tokio::spawn(self.clone().send_impl(request).map(callback));
    }

    async fn send_impl(self, request: Request) -> ResponseResult {
        #![allow(unused)]

        todo!()
        // let grpc: &mut Grpc<Channel> = match &mut inner.grpc {
        //     Some(grpc) => grpc,
        //     None => todo!(),
        // };

        // let codec = ProtobufCodec::new(request.body.descriptor_dyn());

        // let body = grpc
        //     .unary(request.body.into_request(), todo!(), codec)
        //     .await?;

        // Ok(Response {
        //     body: body.into_inner(),
        // })
    }
}
