use anyhow::Result;
use druid::{AppDelegate, Command, DelegateCtx, Env, Handled, Target, WindowHandle, WindowId};

use crate::{
    app::{self, command},
    error::fmt_err,
};

pub(in crate::app) fn build() -> impl AppDelegate<app::State> {
    Delegate
}

struct Delegate;

impl AppDelegate<app::State> for Delegate {
    fn window_added(
        &mut self,
        _: WindowId,
        handle: WindowHandle,
        _: &mut app::State,
        _: &Env,
        _: &mut DelegateCtx,
    ) {
        if let Err(err) = set_window_icon(&handle) {
            tracing::error!("failed to set window icon: {:#}", err);
        }
    }

    fn command(
        &mut self,
        _ctx: &mut DelegateCtx,
        _target: Target,
        cmd: &Command,
        data: &mut app::State,
        _env: &Env,
    ) -> Handled {
        tracing::debug!("Received command: {:?}", cmd);
        if let Some(file) = cmd.get(command::ADD_FILE_ACCEPT) {
            if let Err(err) = data.sidebar.add_from_path(file.path()) {
                data.error = Some(format!("Error loading file: {}", fmt_err(&err)).into());
            } else {
                data.error = None;
            }
            Handled::Yes
        } else if cmd.is(command::OPEN_GITHUB) {
            let _ = open::that(concat!(
                "https://github.com/andrewhickman/lanquetta/tree/",
                env!("VERGEN_GIT_SHA")
            ));
            Handled::Yes
        } else if cmd.is(command::CLOSE_SELECTED_TAB) {
            data.body.close_selected_tab();
            Handled::Yes
        } else if cmd.is(command::SELECT_NEXT_TAB) {
            data.body.select_next_tab();
            Handled::Yes
        } else if cmd.is(command::SELECT_PREV_TAB) {
            data.body.select_prev_tab();
            Handled::Yes
        } else if cmd.is(command::CLEAR) {
            data.body.clear_request_history();
            Handled::Yes
        } else if cmd.is(command::SELECT_OR_CREATE_COMPILE_TAB) {
            data.body
                .select_or_create_compiler_tab(data.sidebar.compile_options());
            Handled::Yes
        } else if cmd.is(command::SELECT_OR_CREATE_REFLECTION_TAB) {
            data.body.select_or_create_reflection_tab();
            Handled::Yes
        } else if let Some((service, options)) = cmd.get(command::SET_SERVICE_OPTIONS) {
            data.body.set_service_options(service, options);
            data.sidebar.set_service_options(service, options);
            Handled::Yes
        } else if let Some(options) = cmd.get(command::SET_COMPILE_OPTIONS) {
            data.sidebar.set_compile_options(options.clone());
            Handled::Yes
        } else if let Some((service, options)) = cmd.get(command::SELECT_OR_CREATE_OPTIONS_TAB) {
            data.body.select_or_create_options_tab(service, options);
            Handled::Yes
        } else if let Some(method) = cmd.get(command::SELECT_OR_CREATE_METHOD_TAB) {
            if let Some(options) = data.sidebar.service_options(method.parent_service()) {
                data.body
                    .select_or_create_method_tab(method, options.clone());
            }
            Handled::Yes
        } else if let Some((service, options)) = cmd.get(command::ADD_SERVICE) {
            data.sidebar.add_service(service.clone(), options.clone());
            Handled::Yes
        } else if let Some(service_index) = cmd.get(command::REMOVE_SERVICE) {
            let service = data.sidebar.remove_service(*service_index);
            data.body.remove_service(service.service());
            Handled::Yes
        } else if let Some(method) = cmd.get(command::CREATE_TAB) {
            if let Some(options) = data.sidebar.service_options(method.parent_service()) {
                data.body.create_method_tab(method, options.clone());
            }
            Handled::Yes
        } else {
            Handled::No
        }
    }
}

#[cfg(windows)]
fn set_window_icon(handle: &WindowHandle) -> Result<()> {
    use druid::{HasRawWindowHandle, RawWindowHandle};

    use windows::Win32::{
        Foundation::{HMODULE, HWND, LPARAM, WPARAM},
        UI::WindowsAndMessaging::{
            LoadImageW, SendMessageW, ICON_BIG, ICON_SMALL, IDI_APPLICATION, IMAGE_ICON,
            LR_DEFAULTSIZE, LR_SHARED, LR_VGACOLOR, WM_SETICON,
        },
    };

    if let RawWindowHandle::Win32(window) = handle.raw_window_handle() {
        unsafe {
            let hwnd = HWND(window.hwnd as isize);

            let image = LoadImageW(
                HMODULE(window.hinstance as isize),
                IDI_APPLICATION,
                IMAGE_ICON,
                0,
                0,
                LR_SHARED | LR_DEFAULTSIZE | LR_VGACOLOR,
            )?;

            // Shown at the top of the window
            SendMessageW(
                hwnd,
                WM_SETICON,
                WPARAM(ICON_SMALL as usize),
                LPARAM(image.0),
            );
            // Shown in the alt+tab window
            SendMessageW(hwnd, WM_SETICON, WPARAM(ICON_BIG as usize), LPARAM(image.0));
        }
    }

    Ok(())
}

#[cfg(not(windows))]
fn set_window_icon(_: &WindowHandle) -> Result<()> {
    Ok(())
}
