use druid::{Command, FileDialogOptions, FileSpec, Selector};
use prost_reflect::{MethodDescriptor, ServiceDescriptor};

use crate::app::sidebar::service::ServiceOptions;

/// Open the source code in a browser
pub const OPEN_GITHUB: Selector = Selector::new("app.open-github");

/// Select the tab with the given service options, or create a new one
pub const SELECT_OR_CREATE_OPTIONS_TAB: Selector<(ServiceDescriptor, ServiceOptions)> =
    Selector::new("app.select-or-create-options-tab");

/// Select the tab with the given method, or create a new one
pub const SELECT_OR_CREATE_METHOD_TAB: Selector<MethodDescriptor> =
    Selector::new("app.select-or-create-method-tab");

/// Set service options
pub const SET_SERVICE_OPTIONS: Selector<(ServiceDescriptor, ServiceOptions)> =
    Selector::new("app.set-service-options");

/// Remove a service
pub const REMOVE_SERVICE: Selector<usize> = Selector::new("app.remove-service");

/// Create a new tab with the given method
pub const CREATE_TAB: Selector<MethodDescriptor> = Selector::new("app.create-tab");

/// Close the selected tab
pub const CLOSE_SELECTED_TAB: Selector = Selector::new("app.close-tab");

/// Close the selected tab
pub const SELECT_NEXT_TAB: Selector = Selector::new("app.select-next-tab");

/// Close the selected tab
pub const SELECT_PREV_TAB: Selector = Selector::new("app.select-prev-tab");

/// Begin connecting to the server
pub const CONNECT: Selector = Selector::new("app.connect");

/// Begin sending a request
pub const SEND: Selector = Selector::new("app.send");

/// Finish sending a request
pub const FINISH: Selector = Selector::new("app.finish");

/// Disconnect from the server
pub const DISCONNECT: Selector = Selector::new("app.disconnect");

/// Clear request history
pub const CLEAR: Selector = Selector::new("app.clear");

/// Add services from a file
pub fn add_file() -> Command {
    const PROTO_DEFINITION_FILE: FileSpec = FileSpec {
        name: "Protobuf definition file",
        extensions: &["proto"],
    };
    const FDSET_FILE: FileSpec = FileSpec {
        name: "File descriptor set",
        extensions: &["bin", "*.*"],
    };

    druid::commands::SHOW_OPEN_PANEL.with(
        FileDialogOptions::new()
            .allowed_types(vec![PROTO_DEFINITION_FILE, FDSET_FILE])
            .default_type(PROTO_DEFINITION_FILE)
            .title("Import services")
            .button_text("Load"),
    )
}
