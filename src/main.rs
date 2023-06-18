#![windows_subsystem = "windows"]
#![allow(clippy::type_complexity)]

use anyhow::{Context, Result};
use tokio::runtime::Runtime;
use tracing_subscriber::{fmt, prelude::*, EnvFilter};

pub fn main() -> Result<()> {
    let filter_layer = EnvFilter::try_from_default_env().or_else(|_| EnvFilter::try_new("info"))?;

    tracing_subscriber::registry()
        .with(fmt::layer())
        .with(filter_layer)
        .init();

    let runtime = Runtime::new().context("failed to initialize the tokio runtime")?;
    let _guard = runtime.enter();

    lanquetta::app::launch()?;

    Ok(())
}
