use druid::widget::{Button, Checkbox, Flex, TextBox};
use druid::{Data, Env, EventCtx, Lens, Target, Widget, WidgetExt};

use crate::app::{command, theme};

#[derive(Debug, Default, Clone, Data, Lens)]
pub(in crate::app) struct State {
    address: String,
    tls: bool,
}

pub(in crate::app) fn build() -> impl Widget<State> {
    let address_textbox = TextBox::new()
        .with_placeholder("localhost:80")
        .lens(State::address)
        .expand_width();
    let tls_checkbox = Checkbox::new("Use TLS").lens(State::tls);
    let send_button = theme::scope(Button::new("Send").on_click(
        |ctx: &mut EventCtx, _: &mut State, _: &Env| {
            ctx.submit_command(command::START_SEND, Target::Global)
        },
    ));

    Flex::row()
        .with_flex_child(address_textbox, 1.0)
        .with_spacer(theme::GUTTER_SIZE)
        .with_child(tls_checkbox)
        .with_spacer(theme::GUTTER_SIZE)
        .with_child(send_button)
        .boxed()
}
