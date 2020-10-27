#![allow(unreachable_code)]

mod app;
mod grpc;
mod json;
mod protobuf;
mod theme;
mod widget;

#[tokio::main]
pub async fn main() -> anyhow::Result<()> {
    env_logger::init();
    app::launch()?;
    Ok(())
}
