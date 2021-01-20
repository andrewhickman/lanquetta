use druid::{
    keyboard_types::Key,
    platform_menus,
    widget::{prelude::*, Controller},
    FileDialogOptions, FileSpec, LocalizedString, MenuDesc, MenuItem, SysMods,
};

use crate::app;

pub const PROTOBUF_FILE_TYPE: FileSpec = FileSpec::new("Protocol buffers file", &["proto"]);

pub struct MenuController;

pub(in crate::app) fn build(data: &app::State) -> MenuDesc<app::State> {
    MenuDesc::empty()
        .append(file_menu(data))
        .append(edit_menu())
        .append(request_menu(data))
        .append(view_menu(data))
        .append(help_menu())
}

fn file_menu(data: &app::State) -> MenuDesc<app::State> {
    MenuDesc::new(LocalizedString::new("common-menu-file-menu"))
        .append(
            MenuItem::new(
                LocalizedString::new("common-menu-file-open"),
                druid::commands::SHOW_OPEN_PANEL
                    .with(FileDialogOptions::new().allowed_types(vec![PROTOBUF_FILE_TYPE])),
            )
            .hotkey(SysMods::Cmd, "o"),
        )
        .append_separator()
        .append(
            MenuItem::new(
                LocalizedString::new("Close Tab"),
                app::command::CLOSE_SELECTED_TAB,
            )
            .hotkey(SysMods::Cmd, "w")
            .disabled_if(|| !has_selected_tab(data)),
        )
        .append(MenuItem::new(
            LocalizedString::new("win-menu-file-exit"),
            druid::commands::QUIT_APP,
        ))
}

fn edit_menu() -> MenuDesc<app::State> {
    MenuDesc::new(LocalizedString::new("common-menu-edit-menu"))
        .append(platform_menus::common::cut())
        .append(platform_menus::common::copy())
        .append(platform_menus::common::paste())
}

fn request_menu(data: &app::State) -> MenuDesc<app::State> {
    MenuDesc::new(LocalizedString::new("Request"))
        .append(
            MenuItem::new(LocalizedString::new("Connect"), app::command::CONNECT)
                .disabled_if(|| !can_connect(data)),
        )
        .append(
            MenuItem::new(LocalizedString::new("Send"), app::command::SEND)
                .hotkey(SysMods::Shift, Key::Enter)
                .disabled_if(|| !can_send(data)),
        )
        .append(
            MenuItem::new(LocalizedString::new("Finish"), app::command::FINISH)
                .disabled_if(|| !can_finish(data)),
        )
        .append(
            MenuItem::new(LocalizedString::new("Disconnect"), app::command::DISCONNECT)
                .disabled_if(|| !can_disconnect(data)),
        )
}

fn view_menu(data: &app::State) -> MenuDesc<app::State> {
    MenuDesc::new(LocalizedString::new("View"))
        .append(
            MenuItem::new(
                LocalizedString::new("Next Tab"),
                app::command::SELECT_NEXT_TAB,
            )
            .hotkey(SysMods::Cmd, Key::Tab)
            .disabled_if(|| !can_select_next_tab(data)),
        )
        .append(
            MenuItem::new(
                LocalizedString::new("Previous Tab"),
                app::command::SELECT_PREV_TAB,
            )
            .hotkey(SysMods::CmdShift, Key::Tab)
            .disabled_if(|| !can_select_prev_tab(data)),
        )
        .append(
            MenuItem::new(
                LocalizedString::new("Clear request history"),
                app::command::CLEAR,
            )
            .hotkey(SysMods::CmdShift, "X")
            .disabled_if(|| !has_selected_tab(data)),
        )
}

fn help_menu() -> MenuDesc<app::State> {
    MenuDesc::new(LocalizedString::new("Help")).append(MenuItem::new(
        LocalizedString::new("GitHub"),
        app::command::OPEN_GITHUB,
    ))
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

impl<W> Controller<app::State, W> for MenuController
where
    W: Widget<app::State>,
{
    fn update(
        &mut self,
        child: &mut W,
        ctx: &mut UpdateCtx,
        old_data: &app::State,
        data: &app::State,
        env: &Env,
    ) {
        if has_selected_tab(old_data) != has_selected_tab(data)
            || can_select_prev_tab(old_data) != can_select_prev_tab(data)
            || can_select_next_tab(old_data) != can_select_next_tab(data)
            || can_connect(old_data) != can_connect(data)
            || can_send(old_data) != can_send(data)
            || can_finish(old_data) != can_finish(data)
            || can_disconnect(old_data) != can_disconnect(data)
        {
            ctx.set_menu(build(data));
        }

        child.update(ctx, old_data, data, env)
    }
}
