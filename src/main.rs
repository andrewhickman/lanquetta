mod app;
mod grpc;
mod json;
mod oneshot;
mod protobuf;
mod theme;
mod widget;

use env_logger::Env;

#[tokio::main]
pub async fn main() -> anyhow::Result<()> {
    env_logger::init_from_env(Env::new().default_filter_or("info"));
    app::launch()?;
    Ok(())
}
