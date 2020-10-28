use std::string::ToString;
use std::{str::FromStr, sync::Arc};

use druid::widget::{Button, Flex, TextBox};
use druid::{Data, Env, EventCtx, Lens, Target, Widget, WidgetExt as _};
use http::Uri;
use once_cell::sync::Lazy;

use crate::app::{command, theme};
use crate::widget::{FormField, ValidationState};

#[derive(Debug, Clone, Data, Lens)]
pub(in crate::app) struct AddressState {
    uri: ValidationState<String, Uri, String>,
}

#[derive(Debug, Clone, Data, Lens)]
pub(in crate::app) struct State {
    address: AddressState,
    #[lens(name = "valid_lens")]
    can_send: bool,
}

pub(in crate::app) fn build() -> Box<dyn Widget<State>> {
    let address_form_field = FormField::new(theme::text_box_scope(
        TextBox::new()
            .with_placeholder("http://localhost:80")
            .expand_width(),
    ));
    let send_button = theme::button_scope(Button::new("Send").on_click(
        |ctx: &mut EventCtx, &mut valid: &mut bool, _: &Env| {
            if valid {
                ctx.submit_command(command::START_SEND.to(Target::Global));
            }
        },
    ))
    .env_scope(|env, &valid: &bool| {
        env.set(theme::DISABLED, !valid);
    });

    Flex::row()
        .with_flex_child(
            address_form_field
                .lens(AddressState::uri)
                .lens(State::address),
            1.0,
        )
        .with_spacer(theme::GUTTER_SIZE)
        .with_child(send_button.lens(State::valid_lens))
        .boxed()
}

static VALIDATE_URI: Lazy<Arc<dyn Fn(&str) -> Result<Uri, String> + Sync + Send>> =
    Lazy::new(|| Arc::new(|s| validate_uri(s)));

fn validate_uri(s: &str) -> Result<Uri, String> {
    let uri = Uri::from_str(s).map_err(|err| err.to_string())?;
    if uri.scheme().is_none() {
        return Err("URI must have scheme".to_owned());
    }
    Ok(uri)
}

impl State {
    pub fn new(address: AddressState, can_send: bool) -> Self {
        State { address, can_send }
    }

    pub fn address_state(&self) -> &AddressState {
        &self.address
    }

    pub fn into_address_state(self) -> AddressState {
        self.address
    }

    pub fn can_send(&self) -> bool {
        self.can_send
    }
}

impl Default for AddressState {
    fn default() -> Self {
        AddressState {
            uri: ValidationState::new(String::default(), VALIDATE_URI.clone()),
        }
    }
}

impl AddressState {
    pub fn is_valid(&self) -> bool {
        self.uri.is_valid()
    }

    pub fn get(&self) -> Option<&Uri> {
        self.uri.result().ok()
    }
}
