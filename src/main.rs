mod app;
mod grpc;
mod json;
mod protobuf;
mod sha;
mod theme;
mod widget;

use std::path::Path;

use directories_next::ProjectDirs;
use env_logger::Env;
use once_cell::sync::Lazy;

#[tokio::main]
pub async fn main() -> anyhow::Result<()> {
    env_logger::init_from_env(Env::new().default_filter_or("debug"));
    app::launch()?;
    Ok(())
}

static PROJECT_DIRS: Lazy<Option<ProjectDirs>> =
    Lazy::new(|| ProjectDirs::from("", "", env!("CARGO_BIN_NAME")));

fn config_dir() -> Option<&'static Path> {
    PROJECT_DIRS.as_ref().map(|dirs| dirs.config_dir())
}

fn data_dir() -> Option<&'static Path> {
    PROJECT_DIRS.as_ref().map(|dirs| dirs.data_dir())
}
