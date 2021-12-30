use std::sync::Arc;

use druid::{
    widget::{prelude::*, Label, LineBreaking, List, ListIter},
    ArcStr, Data, Lens, Widget, WidgetExt,
};
use serde::{Deserialize, Serialize};

use crate::{
    app::{
        command::{REMOVE_SERVICE, SELECT_OR_CREATE_OPTIONS_TAB},
        sidebar::method,
    },
    theme,
    widget::expander,
    widget::{ExpanderData, Icon},
};

#[derive(Debug, Clone, Data, Lens)]
pub(in crate::app) struct State {
    pub index: usize,
    #[data(same_fn = "PartialEq::eq")]
    pub selected: Option<prost_reflect::MethodDescriptor>,
    pub service: ServiceState,
}

#[derive(Debug, Clone, Data, Lens)]
pub(in crate::app) struct ServiceState {
    name: ArcStr,
    #[lens(ignore)]
    methods: Arc<[method::MethodState]>,
    #[lens(ignore)]
    expanded: bool,
    #[data(ignore)]
    #[lens(ignore)]
    service: prost_reflect::ServiceDescriptor,
    #[lens(ignore)]
    options: ServiceOptions,
}

#[derive(Debug, Default, Clone, Data, Lens, Serialize, Deserialize)]
pub struct ServiceOptions {
    pub default_address: String,
}

pub(in crate::app) fn build() -> impl Widget<State> {
    let expander_label = Label::raw()
        .with_font(theme::font::HEADER_ONE)
        .with_line_break_mode(LineBreaking::Clip)
        .lens(ServiceState::name)
        .lens(State::service);

    let open_options_tab: Box<dyn FnMut(&mut EventCtx, &mut State, &Env)> =
        Box::new(move |ctx, data, _| {
            ctx.submit_command(SELECT_OR_CREATE_OPTIONS_TAB.with((
                data.service.service().clone(),
                data.service.options().clone(),
            )));
        });

    let close_expander: Box<dyn FnMut(&mut EventCtx, &mut State, &Env)> =
        Box::new(move |ctx, data, _| {
            ctx.submit_command(REMOVE_SERVICE.with(data.index));
        });

    expander::new(
        expander_label,
        List::new(method::build),
        [
            (Icon::settings(), open_options_tab),
            (Icon::close(), close_expander),
        ]
        .into_iter(),
    )
    .env_scope(|env, data: &State| {
        env.set(theme::EXPANDER_PADDING, 8.0);
        env.set(theme::EXPANDER_CORNER_RADIUS, 0.0);

        let mut bg_color = env.get(druid::theme::BACKGROUND_LIGHT);
        if !data.expanded(env) && data.has_selected() {
            bg_color = theme::color::active(bg_color, env.get(druid::theme::TEXT_COLOR));
        }
        env.set(theme::EXPANDER_BACKGROUND, bg_color);
    })
}

impl State {
    pub fn new(
        selected: Option<prost_reflect::MethodDescriptor>,
        service: ServiceState,
        index: usize,
    ) -> Self {
        State {
            selected,
            service,
            index,
        }
    }

    fn has_selected(&self) -> bool {
        if let Some(selected_method) = &self.selected {
            self.service
                .methods
                .iter()
                .any(|method| selected_method == method.method())
        } else {
            false
        }
    }
}

impl ServiceState {
    pub fn new(
        service: prost_reflect::ServiceDescriptor,
        expanded: bool,
        options: ServiceOptions,
    ) -> Self {
        ServiceState {
            name: service.name().into(),
            methods: service.methods().map(method::MethodState::from).collect(),
            expanded,
            service,
            options,
        }
    }

    pub fn service(&self) -> &prost_reflect::ServiceDescriptor {
        &self.service
    }

    pub fn expanded(&self) -> bool {
        self.expanded
    }

    pub fn options(&self) -> &ServiceOptions {
        &self.options
    }

    pub fn set_options(&mut self, options: ServiceOptions) {
        self.options = options;
    }
}

impl From<prost_reflect::ServiceDescriptor> for ServiceState {
    fn from(service: prost_reflect::ServiceDescriptor) -> Self {
        ServiceState::new(service, true, Default::default())
    }
}

impl ExpanderData for State {
    fn expanded(&self, _: &Env) -> bool {
        self.service.expanded
    }

    fn toggle_expanded(&mut self, _: &Env) {
        self.service.expanded = !self.service.expanded;
    }
}

impl ListIter<method::State> for State {
    fn for_each(&self, mut cb: impl FnMut(&method::State, usize)) {
        for (i, method) in self.service.methods.iter().enumerate() {
            let selected = match &self.selected {
                Some(selected_method) => selected_method == method.method(),
                None => false,
            };
            let state = method::State::new(selected, method.to_owned());
            cb(&state, i);
        }
    }

    fn for_each_mut(&mut self, mut cb: impl FnMut(&mut method::State, usize)) {
        for (i, method) in self.service.methods.iter().enumerate() {
            let selected = match &self.selected {
                Some(selected_method) => selected_method == method.method(),
                None => false,
            };
            let mut state = method::State::new(selected, method.to_owned());
            cb(&mut state, i);

            debug_assert!(selected.same(&state.selected));
            debug_assert!(method.same(&state.method));
        }
    }

    fn data_len(&self) -> usize {
        self.service.methods.len()
    }
}
