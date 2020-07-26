mod app;
mod grpc;
mod widget;

#[tokio::main]
pub async fn main() -> anyhow::Result<()> {
    app::launch()?;
    Ok(())
}
