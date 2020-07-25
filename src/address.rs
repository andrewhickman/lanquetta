use std::sync::Arc;

use druid::widget::{Button, Checkbox, EnvScope, Flex, TextBox};
use druid::{Command, Data, Env, EventCtx, Lens, LocalizedString, Target, Widget, WidgetExt};

use crate::{command, theme};

#[derive(Clone, Data, Lens)]
pub struct AddressState {
    raw_address: String,
    address: Option<Arc<Address>>,
    tls: bool,
}

#[derive(Clone)]
pub struct Address {
    pub host: String,
    pub port: u16,
}

pub fn make_widget() -> impl Widget<AddressState> {
    let address_textbox = EnvScope::new(
        |env: &mut Env, state: &AddressState| state.validate(env),
        TextBox::new()
            .with_placeholder("localhost:80")
            .lens(AddressLens)
            .expand_width(),
    );
    let tls_checkbox = Checkbox::new("Use TLS").lens(AddressState::tls);
    let connect_button = Button::new(LocalizedString::new("Connect")).on_click(
        |ctx: &mut EventCtx, data: &mut AddressState, _: &Env| {
            if let Some(address) = &data.address {
                ctx.submit_command(
                    Command::new(command::CONNECT, (address.clone(), data.tls)),
                    Target::Global,
                );
            }
        },
    );

    Flex::row()
        .with_flex_child(address_textbox, 1.0)
        .with_spacer(16.0)
        .with_child(tls_checkbox)
        .with_spacer(16.0)
        .with_child(connect_button)
}

impl AddressState {
    pub fn new() -> Self {
        AddressState {
            raw_address: String::new(),
            address: None,
            tls: false,
        }
    }

    fn validate(&self, env: &mut Env) {
        if self.is_valid() {
            theme::set_textbox_valid(env);
        } else {
            theme::set_textbox_invalid(env);
        }
    }

    fn is_valid(&self) -> bool {
        self.address.is_some()
    }
}

struct AddressLens;

impl Lens<AddressState, String> for AddressLens {
    fn with<V, F: FnOnce(&String) -> V>(&self, data: &AddressState, f: F) -> V {
        f(&data.raw_address)
    }

    fn with_mut<V, F: FnOnce(&mut String) -> V>(&self, data: &mut AddressState, f: F) -> V {
        let res = f(&mut data.raw_address);
        data.address = parse_address(&data.raw_address);
        res
    }
}

fn parse_address(addr: &str) -> Option<Arc<Address>> {
    let mut split = addr.splitn(2, ':');
    let host = split.next()?.to_owned();
    let port = split.next()?.parse().ok()?;
    Some(Arc::new(Address { host, port }))
}
