use std::{path::PathBuf, time::Duration};

use anyhow::{Context, Result};
use dirs_next::config_dir;
use druid::{
    widget::{prelude::*, Controller},
    Data, Point, Size, TimerToken, Widget, WindowDesc, WindowHandle,
};
use futures::{
    channel::mpsc::{self, UnboundedReceiver},
    StreamExt,
};
use serde::{Deserialize, Serialize};
use tokio::task;

use crate::app::State;

#[derive(Clone, Debug, Default, Deserialize, Serialize, Data)]
pub(in crate::app) struct Config {
    pub window: WindowConfig,
    pub data: State,
}

#[derive(Copy, Clone, Debug, Deserialize, Serialize, Data)]
pub struct WindowConfig {
    state: WindowState,
    size: Size,
    position: Option<Point>,
}

#[derive(Debug)]
pub struct ConfigController {
    sender: mpsc::UnboundedSender<Config>,
    save_timer_token: TimerToken,
    save_task: task::JoinHandle<()>,
}

#[derive(Copy, Clone, Debug, Serialize, Deserialize, Data, PartialEq)]
#[serde(rename_all = "lowercase")]
enum WindowState {
    Maximized,
    Restored,
}

impl Config {
    pub fn load() -> Config {
        Config::try_load().unwrap_or_else(|err| {
            tracing::warn!("Failed to load config: {:?}", err);
            Config::default()
        })
    }

    pub async fn store(config: &Config) {
        if let Err(err) = Config::try_store(config).await {
            tracing::warn!("Failed to store config: {:?}", err);
        }
    }

    fn try_load() -> Result<Config> {
        let path = Config::path()?;
        let text = fs_err::read_to_string(&path)?;
        let config = serde_json::from_str(&text)?;
        tracing::debug!("Loaded config from {}", path.display());
        Ok(config)
    }

    async fn try_store(config: &Config) -> Result<()> {
        let dir = Config::directory()?;
        let path = Config::path()?;
        let text = serde_json::to_string(config)?;

        tokio::fs::create_dir_all(&dir)
            .await
            .with_context(|| format!("failed to create directory `{}`", dir.display()))?;
        tokio::fs::write(&path, text)
            .await
            .with_context(|| format!("failed to write to file `{}`", dir.display()))?;
        tracing::debug!("Stored config to `{}`", path.display());
        Ok(())
    }

    fn directory() -> Result<PathBuf> {
        let mut path = config_dir().context("no config directory found")?;
        path.push(env!("CARGO_BIN_NAME"));
        Ok(path)
    }

    fn path() -> Result<PathBuf> {
        let mut path = Config::directory()?;
        path.push("config.json");
        Ok(path)
    }
}

impl WindowConfig {
    pub fn apply<T>(&self, mut desc: WindowDesc<T>) -> WindowDesc<T>
    where
        T: Data,
    {
        desc = desc.window_size(self.size);
        if let Some(position) = self.position {
            desc = desc.set_position(position);
        }
        desc = desc.set_window_state(match self.state {
            WindowState::Maximized => druid::WindowState::Maximized,
            WindowState::Restored => druid::WindowState::Restored,
        });
        desc
    }

    fn from_handle(handle: &WindowHandle) -> Self {
        WindowConfig {
            size: if let Ok(scale) = handle.get_scale() {
                let size_px = handle.get_size();
                scale.px_to_dp_xy(size_px.width, size_px.height).into()
            } else {
                WindowConfig::default().size
            },
            position: Some(handle.get_position()),
            state: match handle.get_window_state() {
                druid::WindowState::Maximized => WindowState::Maximized,
                druid::WindowState::Minimized | druid::WindowState::Restored => {
                    WindowState::Restored
                }
            },
        }
    }
}

impl Default for WindowConfig {
    fn default() -> Self {
        WindowConfig {
            size: Size::new(1280.0, 768.0),
            position: None,
            state: WindowState::Restored,
        }
    }
}

impl ConfigController {
    const SAVE_INTERVAL: Duration = Duration::from_secs(5);

    pub fn new() -> Self {
        let (sender, receiver) = mpsc::unbounded();

        ConfigController {
            sender,
            save_timer_token: TimerToken::INVALID,
            save_task: task::spawn(Self::run_save(receiver)),
        }
    }

    fn save(&mut self, ctx: &mut EventCtx, data: &State) {
        self.sender
            .unbounded_send(Config {
                window: WindowConfig::from_handle(ctx.window()),
                data: data.clone(),
            })
            .expect("save task exited unexpectedly");
    }

    async fn run_save(mut receiver: UnboundedReceiver<Config>) {
        let mut prev: Option<Config> = None;

        while let Some(mut config) = receiver.next().await {
            while let Ok(Some(buffered_config)) = receiver.try_next() {
                tracing::warn!("Skipping config save because a new version is available");
                config = buffered_config;
            }

            match prev {
                Some(prev) if prev.same(&config) => tracing::debug!("Skipping config save because it is unchanged"),
                _ => Config::store(&config).await,
            }
            prev = Some(config);
        }
    }
}

impl<W> Controller<State, W> for ConfigController
where
    W: Widget<State>,
{
    fn lifecycle(
        &mut self,
        child: &mut W,
        ctx: &mut LifeCycleCtx,
        event: &LifeCycle,
        data: &State,
        env: &Env,
    ) {
        if let LifeCycle::WidgetAdded = event {
            self.save_timer_token = ctx.request_timer(Self::SAVE_INTERVAL);
        }

        child.lifecycle(ctx, event, data, env);
    }

    fn event(
        &mut self,
        child: &mut W,
        ctx: &mut EventCtx,
        event: &Event,
        data: &mut State,
        env: &Env,
    ) {
        match event {
            Event::WindowDisconnected => self.save(ctx, data),
            Event::Timer(token) if token == &self.save_timer_token => {
                self.save(ctx, data);
                self.save_timer_token = ctx.request_timer(Self::SAVE_INTERVAL);
            }
            _ => (),
        }

        child.event(ctx, event, data, env)
    }
}

impl Drop for ConfigController {
    fn drop(&mut self) {
        self.sender.close_channel();
        tracing::debug!("Waiting for config save task to exit");
        tokio::runtime::Handle::current().block_on(&mut self.save_task)
            .expect("save task exited unexpectedly");
        tracing::debug!("Config save task exited");
    }
}
