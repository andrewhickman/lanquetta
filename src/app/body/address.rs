use std::string::ToString;
use std::{str::FromStr, sync::Arc};

use druid::widget::{Button, Flex, CrossAxisAlignment, TextBox};
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
    let address_form_field = FormField::new(theme::text_box_scope(
        TextBox::new()
            .with_placeholder("http://localhost:80")
            .expand_width(),
    ));
    let send_button = theme::button_scope(Button::new("Send").on_click(
        |ctx: &mut EventCtx, _: &mut State, _: &Env| {
            ctx.submit_command(command::START_SEND.to(Target::Global))
        },
    ));

    Flex::row()
        .cross_axis_alignment(CrossAxisAlignment::Baseline)
        .with_flex_child(address_form_field.lens(State::address), 1.0)
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

impl State {
    pub fn get(&self) -> Option<&Uri> {
        self.address.result().ok()
    }
}
