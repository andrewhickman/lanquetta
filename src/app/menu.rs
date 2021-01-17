use druid::{
    platform_menus, FileDialogOptions, FileSpec, LocalizedString, MenuDesc, MenuItem, SysMods,
};

use crate::app;

pub const PROTOBUF_FILE_TYPE: FileSpec = FileSpec::new("Protocol buffers file", &["proto"]);

pub(in crate::app) fn build() -> MenuDesc<app::State> {
    MenuDesc::empty().append(file_menu()).append(edit_menu())
        .append(help_menu())
}

fn file_menu() -> MenuDesc<app::State> {
    MenuDesc::new(LocalizedString::new("common-menu-file-menu"))
        .append(
            MenuItem::new(
                LocalizedString::new("common-menu-file-open"),
                druid::commands::SHOW_OPEN_PANEL
                    .with(FileDialogOptions::new().allowed_types(vec![PROTOBUF_FILE_TYPE])),
            )
            .hotkey(SysMods::Cmd, "o"),
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

fn help_menu() -> MenuDesc<app::State> {
    MenuDesc::new(LocalizedString::new("Help")).append(MenuItem::new(
        LocalizedString::new("GitHub"),
        app::command::OPEN_GITHUB,
    ))
}
