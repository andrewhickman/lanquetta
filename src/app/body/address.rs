use std::string::ToString;
use std::{str::FromStr, sync::Arc};

use druid::widget::{Button, Checkbox, Flex, TextBox};
use druid::{Data, Env, EventCtx, Lens, Target, Widget, WidgetExt as _};
use http::Uri;
use once_cell::sync::Lazy;

use crate::app::{command, theme};
use crate::widget::{FormField, ValidationState};

#[derive(Debug, Clone, Data, Lens)]
pub(in crate::app) struct State {
    address: ValidationState<String, Uri, String>,
    tls: bool,
}

pub(in crate::app) fn build() -> Box<dyn Widget<State>> {
    let address_textbox = TextBox::new()
        .with_placeholder("localhost:80")
        .expand_width();
    let address_form_field = FormField::new(address_textbox);
    let tls_checkbox = Checkbox::new("Use TLS")
        .env_scope(|env, _| env.set(druid::theme::BORDER_LIGHT, theme::color::SUBTLE_ACCENT));
    let send_button = theme::scope(Button::new("Send").on_click(
        |ctx: &mut EventCtx, _: &mut State, _: &Env| {
            ctx.submit_command(command::START_SEND.to(Target::Global))
        },
    ));

    Flex::row()
        .with_flex_child(address_form_field.lens(State::address), 1.0)
        .with_spacer(theme::GUTTER_SIZE)
        .with_child(tls_checkbox.lens(State::tls))
        .with_spacer(theme::GUTTER_SIZE)
        .with_child(send_button)
        .boxed()
}

static VALIDATE_URI: Lazy<Arc<dyn Fn(&str) -> Result<Uri, String> + Sync + Send>> =
    Lazy::new(|| Arc::new(|s| validate_uri(s)));

fn validate_uri(s: &str) -> Result<Uri, String> {
    Uri::from_str(s).map_err(|err| err.to_string())
}

impl Default for State {
    fn default() -> Self {
        State {
            address: ValidationState::new(String::default(), VALIDATE_URI.clone()),
            tls: false,
        }
    }
}
