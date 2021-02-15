#![windows_subsystem = "windows"]

mod app;
mod grpc;
mod json;
mod protobuf;
mod theme;
mod widget;

use tracing_subscriber::{fmt, EnvFilter, prelude::*};

#[tokio::main]
pub async fn main() -> anyhow::Result<()> {
    let filter_layer = EnvFilter::try_from_default_env()
        .or_else(|_| EnvFilter::try_new("info"))?;

    tracing_subscriber::registry()
        .with(fmt::layer())
        .with(filter_layer)
        .init();

    app::launch()?;
    Ok(())
}
