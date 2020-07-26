mod app;
mod grpc;
mod widget;

pub fn main() -> anyhow::Result<()> {
    app::launch()?;
    Ok(())
}
