mod protobuf_codec;

use std::sync::Arc;

use futures::future::FutureExt;
use protobuf::MessageDyn;
use tokio::sync::Mutex;
use tonic::client::Grpc;
use tonic::transport::Channel;
use tonic::IntoRequest;

use self::protobuf_codec::ProtobufCodec;

pub type ResponseResult = Result<Response, Error>;

#[derive(Debug)]
pub struct Request {
    pub body: Box<dyn MessageDyn>,
}

#[derive(Debug)]
pub struct Response {
    pub body: Box<dyn MessageDyn>,
}

pub type Error = anyhow::Error;

#[derive(Clone)]
pub struct Client {
    inner: Arc<Mutex<ClientState>>,
}

struct ClientState {
    grpc: Option<Grpc<Channel>>,
}

impl Client {
    pub fn new() -> Self {
        Client {
            inner: Arc::new(Mutex::new(ClientState { grpc: None })),
        }
    }

    pub fn send(&self, request: Request, callback: impl FnOnce(ResponseResult) + Send + 'static) {
        tokio::spawn(self.clone().send_impl(request).map(callback));
    }

    async fn send_impl(self, request: Request) -> ResponseResult {
        let mut inner = self.inner.lock().await;

        let grpc: &mut Grpc<Channel> = match &mut inner.grpc {
            Some(grpc) => grpc,
            None => todo!(),
        };

        let codec = ProtobufCodec::new(request.body.descriptor_dyn());

        let body = grpc
            .unary(request.body.into_request(), todo!(), codec)
            .await?;

        Ok(Response {
            body: body.into_inner(),
        })
    }
}
