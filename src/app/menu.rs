use druid::{Env, FileDialogOptions, FileSpec, LocalizedString, Menu, MenuItem, SysMods, WindowId, keyboard_types::Key, platform_menus};

use crate::app;

pub const PROTOBUF_FILE_TYPE: FileSpec = FileSpec::new("Protocol buffers file", &["proto"]);

pub(in crate::app) fn build(
    main_window: Option<WindowId>,
    _data: &app::State,
    _env: &Env,
) -> Menu<app::State> {
    let window_id = match main_window {
        Some(id) => id,
        None => return Menu::empty(),
    };

    Menu::empty()
        .entry(file_menu(
            window_id,
        ))
        .entry(edit_menu())
        .entry(request_menu())
        .entry(view_menu())
        .entry(help_menu())
}

fn file_menu(main_window: WindowId) -> Menu<app::State> {
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
            MenuItem::new(LocalizedString::new("Close Tab"))
                .command(app::command::CLOSE_SELECTED_TAB)
                .hotkey(SysMods::Cmd, "w")
                .enabled_if(|data, _| has_selected_tab(data)),
        )
        .entry(
            MenuItem::new(LocalizedString::new("win-menu-file-exit"))
                .command(druid::commands::CLOSE_WINDOW.to(main_window)),
        )
}

fn edit_menu() -> Menu<app::State> {
    Menu::new(LocalizedString::new("common-menu-edit-menu"))
        .entry(platform_menus::common::cut())
        .entry(platform_menus::common::copy())
        .entry(platform_menus::common::paste())
}

fn request_menu() -> Menu<app::State> {
    Menu::new(LocalizedString::new("Request"))
        .entry(
            MenuItem::new(LocalizedString::new("Connect"))
                .command(app::command::CONNECT)
                .enabled_if(|data, _| can_connect(data)),
        )
        .entry(
            MenuItem::new(LocalizedString::new("Send"))
                .command(app::command::SEND)
                .hotkey(SysMods::Shift, Key::Enter)
                .enabled_if(|data, _| can_send(data)),
        )
        .entry(
            MenuItem::new(LocalizedString::new("Finish"))
                .command(app::command::FINISH)
                .enabled_if(|data, _| can_finish(data)),
        )
        .entry(
            MenuItem::new(LocalizedString::new("Disconnect"))
                .command(app::command::DISCONNECT)
                .enabled_if(|data, _| can_disconnect(data)),
        )
}

fn view_menu() -> Menu<app::State> {
    Menu::new(LocalizedString::new("View"))
        .entry(
            MenuItem::new(LocalizedString::new("Next Tab"))
                .command(app::command::SELECT_NEXT_TAB)
                .hotkey(SysMods::Cmd, Key::Tab)
                .enabled_if(|data, _| can_select_next_tab(data)),
        )
        .entry(
            MenuItem::new(LocalizedString::new("Previous Tab"))
                .command(app::command::SELECT_PREV_TAB)
                .hotkey(SysMods::CmdShift, Key::Tab)
                .enabled_if(|data, _| can_select_prev_tab(data)),
        )
        .entry(
            MenuItem::new(LocalizedString::new("Clear request history"))
                .command(app::command::CLEAR)
                .hotkey(SysMods::CmdShift, "X")
                .enabled_if(|data, _| has_selected_tab(data)),
        )
}

fn help_menu() -> Menu<app::State> {
    Menu::new(LocalizedString::new("Help"))
        .entry(MenuItem::new(LocalizedString::new("GitHub")).command(app::command::OPEN_GITHUB))
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
