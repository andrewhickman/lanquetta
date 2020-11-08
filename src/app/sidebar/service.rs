use std::sync::Arc;

use druid::{
    widget::{prelude::*, List, ListIter},
    ArcStr, Data, FontDescriptor, FontFamily, Lens, Widget, WidgetExt,
};

use crate::{
    app::{command::REMOVE_SERVICE, sidebar::method},
    protobuf::{ProtobufMethod, ProtobufService},
    theme,
    widget::Expander,
    widget::ExpanderData,
};

#[derive(Debug, Clone, Data, Lens)]
pub(in crate::app) struct State {
    pub index: usize,
    pub selected: Option<ProtobufMethod>,
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
    service: ProtobufService,
}

pub(in crate::app) fn build(sidebar_id: WidgetId) -> Box<dyn Widget<State>> {
    Expander::new(
        move |ctx, data: &mut State, _| {
            ctx.submit_command(REMOVE_SERVICE.with(data.index).to(sidebar_id));
        },
        List::new(method::build),
    )
    .env_scope(|env, data: &State| {
        env.set(theme::EXPANDER_PADDING, 8.0);
        env.set(theme::EXPANDER_CORNER_RADIUS, 0.0);
        env.set(
            theme::EXPANDER_LABEL_FONT,
            FontDescriptor::new(FontFamily::SANS_SERIF).with_size(18.0),
        );

        let mut bg_color = env.get(theme::SIDEBAR_BACKGROUND);
        if !data.expanded(env) && data.has_selected() {
            bg_color = theme::color::active(bg_color, env.get(druid::theme::LABEL_COLOR));
        }
        env.set(theme::EXPANDER_BACKGROUND, bg_color);
    })
    .boxed()
}

impl State {
    pub fn new(selected: Option<ProtobufMethod>, service: ServiceState, index: usize) -> Self {
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
                .any(|method| selected_method.same(method.method()))
        } else {
            false
        }
    }
}

impl ServiceState {
    pub fn new(service: ProtobufService, expanded: bool) -> Self {
        ServiceState {
            name: service.name().into(),
            methods: service.methods().map(method::MethodState::from).collect(),
            expanded,
            service,
        }
    }

    pub fn service(&self) -> &ProtobufService {
        &self.service
    }

    pub fn expanded(&self) -> bool {
        self.expanded
    }
}

impl From<ProtobufService> for ServiceState {
    fn from(service: ProtobufService) -> Self {
        ServiceState::new(service, true)
    }
}

impl ExpanderData for State {
    fn expanded(&self, _: &Env) -> bool {
        self.service.expanded
    }

    fn toggle_expanded(&mut self, _: &Env) {
        self.service.expanded = !self.service.expanded;
    }

    fn with_label<V>(&self, f: impl FnOnce(&ArcStr) -> V) -> V {
        f(&self.service.name)
    }

    fn with_label_mut<V>(&mut self, f: impl FnOnce(&mut ArcStr) -> V) -> V {
        f(&mut self.service.name)
    }

    fn can_close(&self) -> bool {
        true
    }
}

impl ListIter<method::State> for State {
    fn for_each(&self, mut cb: impl FnMut(&method::State, usize)) {
        for (i, method) in self.service.methods.iter().enumerate() {
            let selected = match &self.selected {
                Some(selected_method) => selected_method.same(method.method()),
                None => false,
            };
            let state = method::State::new(selected, method.to_owned());
            cb(&state, i);
        }
    }

    fn for_each_mut(&mut self, mut cb: impl FnMut(&mut method::State, usize)) {
        for (i, method) in self.service.methods.iter().enumerate() {
            let selected = match &self.selected {
                Some(selected_method) => selected_method.same(method.method()),
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
