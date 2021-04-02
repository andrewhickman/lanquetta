use druid::{
    keyboard_types::Key, menu, Env, FileDialogOptions, FileSpec, LocalizedString, Menu, MenuItem,
    SysMods, WindowId,
};

use crate::app;

pub const PROTOBUF_FILE_TYPE: FileSpec = FileSpec::new("Protocol buffers file", &["proto"]);

pub(in crate::app) fn build(
    _window: Option<WindowId>,
    _data: &app::State,
    _env: &Env,
) -> Menu<app::State> {
    Menu::empty()
        .entry(file_menu())
        .entry(edit_menu())
        .entry(request_menu())
        .entry(view_menu())
        .entry(help_menu())
}

fn file_menu() -> Menu<app::State> {
    Menu::new(LocalizedString::new("common-menu-file-menu"))
        .entry(
            MenuItem::new(LocalizedString::new("common-menu-file-open"))
                .command(
                    druid::commands::SHOW_OPEN_PANEL
                        .with(FileDialogOptions::new().allowed_types(vec![PROTOBUF_FILE_TYPE])),
                )
                .hotkey(SysMods::Cmd, "o"),
        )
        .separator()
        .entry(
            MenuItem::new("Close Tab")
                .command(app::command::CLOSE_SELECTED_TAB)
                .hotkey(SysMods::Cmd, "w")
                .enabled_if(|data, _| has_selected_tab(data)),
        )
        .entry(menu::sys::win::file::close())
}

fn edit_menu() -> Menu<app::State> {
    Menu::new(LocalizedString::new("common-menu-edit-menu"))
        .entry(menu::sys::common::cut())
        .entry(menu::sys::common::copy())
        .entry(menu::sys::common::paste())
}

fn request_menu() -> Menu<app::State> {
    Menu::new("Request")
        .entry(
            MenuItem::new("Connect")
                .command(app::command::CONNECT)
                .enabled_if(|data, _| can_connect(data)),
        )
        .entry(
            MenuItem::new("Send")
                .command(app::command::SEND)
                .hotkey(SysMods::Shift, Key::Enter)
                .enabled_if(|data, _| can_send(data)),
        )
        .entry(
            MenuItem::new("Finish")
                .command(app::command::FINISH)
                .enabled_if(|data, _| can_finish(data)),
        )
        .entry(
            MenuItem::new("Disconnect")
                .command(app::command::DISCONNECT)
                .enabled_if(|data, _| can_disconnect(data)),
        )
}

fn view_menu() -> Menu<app::State> {
    Menu::new("View")
        .entry(
            MenuItem::new("Next Tab")
                .command(app::command::SELECT_NEXT_TAB)
                .hotkey(SysMods::Cmd, Key::Tab)
                .enabled_if(|data, _| can_select_next_tab(data)),
        )
        .entry(
            MenuItem::new("Previous Tab")
                .command(app::command::SELECT_PREV_TAB)
                .hotkey(SysMods::CmdShift, Key::Tab)
                .enabled_if(|data, _| can_select_prev_tab(data)),
        )
        .entry(
            MenuItem::new("Clear request history")
                .command(app::command::CLEAR)
                .hotkey(SysMods::CmdShift, "X")
                .enabled_if(|data, _| has_selected_tab(data)),
        )
}

fn help_menu() -> Menu<app::State> {
    Menu::new("Help").entry(MenuItem::new("GitHub").command(app::command::OPEN_GITHUB))
}

fn has_selected_tab(data: &app::State) -> bool {
    data.body.selected_tab().is_some()
}

fn can_select_next_tab(data: &app::State) -> bool {
    data.body.selected_tab() != data.body.last_tab()
}

fn can_select_prev_tab(data: &app::State) -> bool {
    data.body.selected_tab() != data.body.first_tab()
}

fn can_connect(data: &app::State) -> bool {
    data.body
        .with_selected_address(|address| address.can_connect())
        .unwrap_or(false)
}

fn can_send(data: &app::State) -> bool {
    data.body
        .with_selected_address(|address| address.can_send())
        .unwrap_or(false)
}

fn can_finish(data: &app::State) -> bool {
    data.body
        .with_selected_address(|address| address.can_finish())
        .unwrap_or(false)
}

fn can_disconnect(data: &app::State) -> bool {
    data.body
        .with_selected_address(|address| address.can_disconnect())
        .unwrap_or(false)
}
