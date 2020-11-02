use druid::{FileDialogOptions, FileSpec, LocalizedString, MenuDesc, MenuItem, SysMods};

use crate::app;

pub const PROTOBUF_FILE_TYPE: FileSpec = FileSpec::new("Protocol buffers file", &["proto"]);

pub(in crate::app) fn build() -> MenuDesc<app::State> {
    MenuDesc::empty().append(file_menu())
}

fn file_menu() -> MenuDesc<app::State> {
    MenuDesc::new(LocalizedString::new("common-menu-file-menu")).append(
        MenuItem::new(
            LocalizedString::new("common-menu-file-open"),
            druid::commands::SHOW_OPEN_PANEL
                .with(FileDialogOptions::new().allowed_types(vec![PROTOBUF_FILE_TYPE])),
        )
        .hotkey(SysMods::Cmd, "o"),
    )
}
