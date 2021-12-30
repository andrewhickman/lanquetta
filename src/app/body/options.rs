use druid::{
    widget::{Controller, Either, Flex, Label, TextBox},
    Data, Env, Lens, UpdateCtx, Widget, WidgetExt,
};
use http::Uri;
use prost_reflect::ServiceDescriptor;

use crate::{
    app::{body::VALIDATE_URI, command::SET_SERVICE_OPTIONS, sidebar::service::ServiceOptions},
    theme,
    widget::{Empty, FormField, ValidationState},
};

type AddressValidationState = ValidationState<String, Uri, String>;

#[derive(Debug, Clone, Data, Lens)]
pub struct OptionsTabState {
    #[data(same_fn = "PartialEq::eq")]
    #[lens(ignore)]
    service: ServiceDescriptor,
    default_address: AddressValidationState,
}

pub struct OptionsTabController;

pub fn build_body() -> impl Widget<OptionsTabState> {
    let address_textbox = FormField::new(theme::text_box_scope(
        TextBox::new()
            .with_placeholder("http://localhost:80")
            .expand_width(),
    ));

    let error_label =
        theme::error_label_scope(Label::dynamic(|data: &AddressValidationState, _| {
            data.result().err().cloned().unwrap_or_default()
        }));
    let error = Either::new(
        |data: &AddressValidationState, _| !data.is_pristine_or_valid(),
        error_label,
        Empty,
    )
    .expand_width();

    Flex::column()
        .with_child(
            Label::new("Default address")
                .with_font(theme::font::HEADER_TWO)
                .align_left(),
        )
        .with_spacer(theme::BODY_SPACER)
        .with_child(address_textbox)
        .with_child(error)
        .must_fill_main_axis(true)
        .lens(OptionsTabState::default_address)
        .padding(theme::BODY_PADDING)
        .expand_height()
        .controller(OptionsTabController)
}

impl OptionsTabState {
    pub fn new(service: ServiceDescriptor, options: ServiceOptions) -> Self {
        OptionsTabState {
            service,
            default_address: ValidationState::new(
                options
                    .default_address
                    .map(|s| s.to_string())
                    .unwrap_or_default(),
                VALIDATE_URI.clone(),
            ),
        }
    }

    pub fn label(&self) -> String {
        format!("{} options", self.service.name())
    }

    pub fn service(&self) -> &ServiceDescriptor {
        &self.service
    }

    pub fn service_options(&self) -> Option<ServiceOptions> {
        self.default_address
            .result()
            .ok()
            .map(|default_address| ServiceOptions {
                default_address: Some(default_address.clone()),
            })
    }
}

impl<W> Controller<OptionsTabState, W> for OptionsTabController
where
    W: Widget<OptionsTabState>,
{
    fn update(
        &mut self,
        child: &mut W,
        ctx: &mut UpdateCtx,
        old_data: &OptionsTabState,
        data: &OptionsTabState,
        env: &Env,
    ) {
        if !old_data.same(data) {
            if let Some(service_options) = dbg!(data.service_options()) {
                ctx.submit_command(
                    SET_SERVICE_OPTIONS.with((data.service.clone(), service_options)),
                );
            }
            child.update(ctx, old_data, data, env);
        }
    }
}
