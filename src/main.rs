mod app;
mod grpc;

pub fn main() -> anyhow::Result<()> {
    app::launch()?;
    Ok(())
}
