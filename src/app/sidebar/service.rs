use std::sync::Arc;

use druid::{
    widget::{CrossAxisAlignment, Either, Flex, Label, LineBreaking, List, ListIter, Painter},
    ArcStr, Data, FontDescriptor, FontFamily, Lens, RenderContext, Widget, WidgetExt as _,
};
use serde::{Deserialize, Serialize};

use crate::{
    app::sidebar::method,
    protobuf::{ProtobufMethod, ProtobufService},
    theme,
    widget::{Empty, Icon},
};

#[derive(Debug, Clone, Data, Lens)]
pub(in crate::app) struct State {
    pub selected: Option<ProtobufMethod>, // TODO generate from selected tab in body
    pub service: ServiceState,
}

#[derive(Debug, Clone, Data, Lens, Serialize, Deserialize)]
pub(in crate::app) struct ServiceState {
    name: ArcStr,
    #[serde(skip, default = "arc_new")]
    methods: Arc<[method::MethodState]>,
    expanded: bool,
}

fn arc_new() -> Arc<[method::MethodState]> {
    Arc::new([])
}

pub(in crate::app) fn build() -> Box<dyn Widget<State>> {
    let service_label = Label::raw()
        .with_font(FontDescriptor::new(FontFamily::SANS_SERIF))
        .with_text_size(18.0)
        .with_line_break_mode(LineBreaking::Clip)
        .padding(theme::GUTTER_SIZE / 2.0)
        .expand_width()
        .lens(ServiceState::name);
    let service_icon = Either::new(
        |state: &ServiceState, _| state.expanded,
        Icon::expand_more(),
        Icon::chevron_right(),
    );
    let service = Flex::row()
        .with_child(service_icon)
        .with_flex_child(service_label, 1.0)
        .lens(State::service)
        .background(Painter::new(|ctx, data: &State, env| {
            let mut color = env.get(theme::SIDEBAR_BACKGROUND);
            if !data.service.expanded && data.has_selected() {
                color = theme::color::active(color, env.get(druid::theme::LABEL_COLOR));
            }
            if ctx.is_active() {
                color = theme::color::active(color, env.get(druid::theme::LABEL_COLOR));
            } else if ctx.is_hot() {
                color = theme::color::hot(color, env.get(druid::theme::LABEL_COLOR));
            }
            let bounds = ctx.size().to_rect();
            ctx.fill(bounds, &color);
        }))
        .on_click(|_, data, _| data.service.expanded = !data.service.expanded);

    let methods = Either::new(
        |state: &State, _| state.service.expanded,
        List::new(method::build),
        Empty,
    );

    Flex::column()
        .cross_axis_alignment(CrossAxisAlignment::Start)
        .with_child(service)
        .with_child(methods)
        .boxed()
}

impl State {
    pub fn new(selected: Option<ProtobufMethod>, service: ServiceState) -> Self {
        State { selected, service }
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

impl From<ProtobufService> for ServiceState {
    fn from(service: ProtobufService) -> Self {
        ServiceState {
            name: service.name().into(),
            methods: service.methods().map(method::MethodState::from).collect(),
            expanded: true,
        }
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
