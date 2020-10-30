use std::path::PathBuf;

use anyhow::{Context, Result};
use druid::{
    commands::CLOSE_WINDOW,
    widget::{prelude::*, Controller},
    Point, Size, Widget, WidgetExt as _, WindowDesc, WindowHandle,
};
use fs_err::{create_dir_all, read_to_string, write};
use serde::{Deserialize, Serialize};

use crate::{app::State, config_dir};

#[derive(Debug, Default, Serialize, Deserialize)]
pub struct Config {
    window: WindowConfig,
}

#[derive(Copy, Clone, Debug, Serialize, Deserialize)]
pub struct WindowConfig {
    state: WindowState,
    size: Size,
    position: Option<Point>,
}

struct ConfigController {
    config: Config,
}

#[derive(Copy, Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
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

    pub(in crate::app) fn make_window<W, F>(self, root: F) -> WindowDesc<State>
    where
        W: Widget<State> + 'static,
        F: FnOnce() -> W + 'static,
    {
        let window_config = self.window;
        let window_desc = WindowDesc::new(|| root().controller(ConfigController { config: self }));
        window_config.apply(window_desc)
    }

    fn try_load() -> Result<Config> {
        let path = Config::path()?;
        let text = read_to_string(path)?;
        let config = toml::from_str(&text)?;
        Ok(config)
    }

    fn try_store(config: &Config) -> Result<()> {
        let path = Config::path()?;
        let text = toml::to_string(config)?;
        create_dir_all(path.parent().unwrap())?;
        write(path, text)?;
        Ok(())
    }

    fn path() -> Result<PathBuf> {
        Ok(config_dir()
            .context("config directory not found")?
            .join("config.toml"))
    }
}

impl WindowConfig {
    fn apply(&self, mut desc: WindowDesc<State>) -> WindowDesc<State> {
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

    fn update(&mut self, handle: &WindowHandle) {
        if let Ok(scale) = handle.get_scale() {
            let size_px = handle.get_size();
            self.size = scale.px_to_dp_xy(size_px.width, size_px.height).into();
        }

        self.position = Some(handle.get_position());
        self.state = match handle.get_window_state() {
            druid::WindowState::MAXIMIZED => WindowState::Maximized,
            druid::WindowState::MINIMIZED | druid::WindowState::RESTORED => WindowState::Restored,
        };
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
                self.config.window.update(ctx.window());
                Config::store(&self.config);
            }
        }

        child.event(ctx, event, data, env)
    }
}
