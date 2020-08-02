mod serde_codec;

use std::sync::Arc;

use futures::future::FutureExt;
use tokio::sync::Mutex;
use tonic::client::Grpc;
use tonic::transport::Channel;

pub type ResponseResult = Result<Response, Error>;

#[derive(Debug)]
pub struct Request {
    pub body: String,
}

#[derive(Debug)]
pub struct Response {
    pub body: String,
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
        let item: protobuf::well_known_types::Value =
            serde_json::from_str(&request.body).map_err(anyhow::Error::from)?;
        Ok(Response {
            body: serde_json::to_string(&item).unwrap(),
        })
    }
}
