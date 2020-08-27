mod app;
mod grpc;
mod protobuf;
mod theme;
mod widget;

#[tokio::main]
pub async fn main() -> anyhow::Result<()> {
    app::launch()?;
    Ok(())
}
