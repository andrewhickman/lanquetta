use std::path::PathBuf;

use anyhow::{Context, Result};
use dirs_next::config_dir;
use druid::{
    commands::CLOSE_WINDOW,
    widget::{prelude::*, Controller},
    Data, Point, Size, Widget, WindowDesc, WindowHandle,
};
use fs_err::{create_dir_all, read_to_string, write};
use serde::{Deserialize, Serialize};

use crate::app::State;

#[derive(Debug, Default, Deserialize, Serialize)]
pub(in crate::app) struct Config {
    pub window: WindowConfig,
    pub data: State,
}

#[derive(Copy, Clone, Debug, Deserialize, Serialize)]
pub struct WindowConfig {
    state: WindowState,
    size: Size,
    position: Option<Point>,
}

#[derive(Debug)]
pub struct ConfigController;

#[derive(Copy, Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
enum WindowState {
    Maximized,
    Restored,
}

impl Config {
    pub fn load() -> Config {
        match Config::try_load() {
            Ok(config) => {
                log::debug!("Loaded config {:#?}", config);
                config
            }
            Err(err) => {
                log::warn!("Failed to load config: {:?}", err);
                Config::default()
            }
        }
    }

    pub fn store(config: &Config) {
        if let Err(err) = Config::try_store(config) {
            log::warn!("Failed to store config: {:?}", err);
        }
    }

    fn try_load() -> Result<Config> {
        let path = Config::path()?;
        let text = read_to_string(path)?;
        let config = serde_json::from_str(&text)?;
        Ok(config)
    }

    fn try_store(config: &Config) -> Result<()> {
        let path = Config::path()?;
        let text = serde_json::to_string(config)?;
        create_dir_all(path.parent().unwrap())?;
        write(path, text)?;
        Ok(())
    }

    fn path() -> Result<PathBuf> {
        let mut path = config_dir().context("no config directory found")?;
        path.push(env!("CARGO_BIN_NAME"));
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
            WindowState::Maximized => druid::WindowState::MAXIMIZED,
            WindowState::Restored => druid::WindowState::RESTORED,
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
                druid::WindowState::MAXIMIZED => WindowState::Maximized,
                druid::WindowState::MINIMIZED | druid::WindowState::RESTORED => {
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

impl<W> Controller<State, W> for ConfigController
where
    W: Widget<State>,
{
    fn event(
        &mut self,
        child: &mut W,
        ctx: &mut EventCtx,
        event: &Event,
        data: &mut State,
        env: &Env,
    ) {
        if let Event::Command(command) = event {
            if command.is(CLOSE_WINDOW) {
                Config::store(&Config {
                    window: WindowConfig::from_handle(ctx.window()),
                    data: data.clone(),
                });
            }
        }

        child.event(ctx, event, data, env)
    }
}
