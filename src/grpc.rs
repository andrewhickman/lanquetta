use futures::future::FutureExt;

pub type ResponseResult = Result<Response, Error>;

#[derive(Debug)]
pub struct Error;

#[derive(Debug)]
pub struct Response;

#[derive(Debug)]
pub struct Request;

#[derive(Debug, Clone)]
pub struct Client {}

impl Client {
    pub fn new() -> Self {
        Client {}
    }

    pub fn send(&self, request: Request, callback: impl FnOnce(ResponseResult) + Send + 'static) {
        tokio::spawn(self.clone().send_impl(request).map(callback));
    }

    async fn send_impl(self, _request: Request) -> ResponseResult {
        Ok(Response)
    }
}
