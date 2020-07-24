use druid::widget::{Checkbox, CrossAxisAlignment, EnvScope, Flex, MainAxisAlignment, TextBox};
use druid::{Data, Env, Lens, Widget, WidgetExt};

use crate::theme;

#[derive(Clone, Data, Lens)]
pub struct ChannelState {
    raw_address: String,
    address: Option<(String, u16)>,
    tls: bool,
}

pub fn make_widget() -> impl Widget<ChannelState> {
    let address_textbox = TextBox::new()
        .with_placeholder("localhost:80")
        .lens(lens::Address)
        .expand_width();
    let tls_checkbox = Checkbox::new("Use TLS").lens(ChannelState::tls);

    EnvScope::new(
        |env: &mut Env, state: &ChannelState| state.validate(env),
        Flex::column()
            .main_axis_alignment(MainAxisAlignment::Start)
            .cross_axis_alignment(CrossAxisAlignment::Start)
            .must_fill_main_axis(true)
            .with_flex_child(
                Flex::row()
                    .with_flex_child(address_textbox, 1.0)
                    .with_spacer(16.0)
                    .with_child(tls_checkbox),
                1.0,
            )
            .with_flex_spacer(1.0),
    )
}

impl ChannelState {
    pub fn new() -> Self {
        ChannelState {
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

mod lens {
    use druid::Lens;

    pub(super) struct Address;

    impl Lens<super::ChannelState, String> for Address {
        fn with<V, F: FnOnce(&String) -> V>(&self, data: &super::ChannelState, f: F) -> V {
            f(&data.raw_address)
        }

        fn with_mut<V, F: FnOnce(&mut String) -> V>(
            &self,
            data: &mut super::ChannelState,
            f: F,
        ) -> V {
            let res = f(&mut data.raw_address);
            data.address = parse_address(&data.raw_address);
            res
        }
    }

    fn parse_address<'a>(addr: &'a str) -> Option<(String, u16)> {
        let mut split = addr.splitn(2, ':');
        let host = split.next()?.to_owned();
        let port = split.next()?.parse().ok()?;
        Some((host, port))
    }
}
