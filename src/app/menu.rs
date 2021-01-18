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
            .disabled_if(|| !can_close_selected_tab(data)),
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
}

fn help_menu() -> MenuDesc<app::State> {
    MenuDesc::new(LocalizedString::new("Help")).append(MenuItem::new(
        LocalizedString::new("GitHub"),
        app::command::OPEN_GITHUB,
    ))
}

fn can_close_selected_tab(data: &app::State) -> bool {
    data.body.selected_tab().is_some()
}

fn can_select_next_tab(data: &app::State) -> bool {
    data.body.selected_tab() != data.body.last_tab()
}

fn can_select_prev_tab(data: &app::State) -> bool {
    data.body.selected_tab() != data.body.first_tab()
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
        if can_close_selected_tab(old_data) != can_close_selected_tab(data)
            || can_select_prev_tab(old_data) != can_select_prev_tab(data)
            || can_select_next_tab(old_data) != can_select_next_tab(data)
        {
            ctx.set_menu(build(data));
        }

        child.update(ctx, old_data, data, env)
    }
}
