use std::{
    collections::BTreeMap,
    convert::{TryFrom, TryInto},
};

use anyhow::{Context, Error, Result};
use base64::{
    alphabet,
    engine::{GeneralPurpose, GeneralPurposeConfig},
    Engine,
};
use druid::piet::TextStorage;
use prost_reflect::DescriptorPool;
use prost_reflect::{prost::Message, prost_types::FileDescriptorSet};
use serde::{
    de::{self, Deserializer},
    ser::{self, Serializer},
    Deserialize, Serialize,
};

use crate::{
    app::{self, sidebar::service::ServiceOptions},
    json::JsonText,
    widget::{TabId, TabsData},
};

const STANDARD: GeneralPurpose =
    GeneralPurpose::new(&alphabet::STANDARD, GeneralPurposeConfig::new());

impl Serialize for app::State {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        AppState::try_from(self)
            .map_err(ser::Error::custom)?
            .serialize(serializer)
    }
}

impl<'de> Deserialize<'de> for app::State {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        AppState::deserialize(deserializer)?
            .try_into()
            .map_err(de::Error::custom)
    }
}

#[derive(Debug, Serialize, Deserialize)]
struct AppState {
    file_descriptor_sets: Vec<String>,
    services: Vec<AppServiceState>,
    body: AppBodyState,
}

#[derive(Debug, Serialize, Deserialize, PartialEq)]
struct AppServiceRef {
    file_set: usize,
    service: usize,
}

#[derive(Debug, Serialize, Deserialize)]
struct AppServiceState {
    #[serde(flatten)]
    idx: AppServiceRef,
    expanded: bool,
    options: ServiceOptions,
}

#[derive(Debug, Serialize, Deserialize)]
struct AppBodyState {
    tabs: Vec<AppBodyTabState>,
    selected: Option<usize>,
}

#[derive(Debug, Serialize, Deserialize)]
struct AppBodyTabState {
    #[serde(flatten)]
    kind: AppBodyTabKind,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
#[allow(clippy::large_enum_variant)]
enum AppBodyTabKind {
    Method {
        #[serde(flatten)]
        idx: AppServiceRef,
        method: usize,
        address: String,
        request: String,
        stream: app::body::StreamState,
        options: ServiceOptions,
    },
    Options {
        #[serde(flatten)]
        idx: AppServiceRef,
    },
}

impl<'a> TryFrom<&'a app::State> for AppState {
    type Error = Error;

    fn try_from(data: &'a app::State) -> Result<Self, Self::Error> {
        let mut file_descriptor_sets = Vec::with_capacity(data.sidebar.services().len());

        let services = data
            .sidebar
            .services()
            .iter()
            .map(|service| {
                let file_set = get_or_insert_file_set(
                    &mut file_descriptor_sets,
                    service.service().parent_pool(),
                )?;
                Ok(AppServiceState {
                    idx: AppServiceRef {
                        file_set,
                        service: service.service().index(),
                    },
                    expanded: service.expanded(),
                    options: service.options().clone(),
                })
            })
            .collect::<Result<Vec<_>>>()?;

        let body = AppBodyState {
            tabs: data
                .body
                .tabs()
                .map(|(_, tab)| {
                    let kind = match tab {
                        app::body::TabState::Method(method) => {
                            let file_set = get_or_insert_file_set(
                                &mut file_descriptor_sets,
                                method.method().parent_pool(),
                            )?;

                            AppBodyTabKind::Method {
                                idx: AppServiceRef {
                                    file_set,
                                    service: method.method().parent_service().index(),
                                },
                                method: method.method().index(),
                                address: method.address().text().to_owned(),
                                request: method.request().text().as_str().to_owned(),
                                stream: method.stream().clone(),
                                options: method.service_options().clone(),
                            }
                        }
                        app::body::TabState::Options(options) => {
                            let file_set = get_or_insert_file_set(
                                &mut file_descriptor_sets,
                                options.service().parent_pool(),
                            )?;

                            AppBodyTabKind::Options {
                                idx: AppServiceRef {
                                    file_set,
                                    service: options.service().index(),
                                },
                            }
                        }
                    };

                    Ok(AppBodyTabState { kind })
                })
                .collect::<Result<Vec<_>>>()?,
            selected: data
                .body
                .selected()
                .and_then(|selected| data.body.tabs().position(|(id, _)| id == selected)),
        };

        let file_descriptor_sets = file_descriptor_sets
            .into_iter()
            .map(|f| {
                let file_set = FileDescriptorSet {
                    file: f.file_descriptor_protos().cloned().collect(),
                };
                STANDARD.encode(file_set.encode_to_vec())
            })
            .collect();

        Ok(AppState {
            file_descriptor_sets,
            services,
            body,
        })
    }
}

impl TryInto<app::State> for AppState {
    type Error = Error;

    fn try_into(self) -> Result<app::State, Self::Error> {
        let AppState {
            file_descriptor_sets,
            services,
            body,
        } = self;

        let file_descriptor_sets = file_descriptor_sets
            .into_iter()
            .map(|b64| {
                let bytes = STANDARD.decode(b64)?;
                anyhow::Ok(DescriptorPool::decode(bytes.as_ref())?)
            })
            .collect::<Result<Vec<_>, _>>()?;

        Ok(app::State {
            sidebar: services
                .iter()
                .map(|service| {
                    Ok(app::sidebar::service::ServiceState::new(
                        get_service(&file_descriptor_sets, &service.idx)?,
                        service.expanded,
                        service.options.clone(),
                    ))
                })
                .collect::<Result<app::sidebar::ServiceListState>>()?,
            body: body.into_state(&file_descriptor_sets, &services)?,
            error: None,
        })
    }
}

impl AppBodyState {
    fn into_state(
        self,
        file_sets: &[prost_reflect::DescriptorPool],
        services: &[AppServiceState],
    ) -> Result<app::body::State> {
        let tabs = self
            .tabs
            .into_iter()
            .map(|tab| match tab.kind {
                AppBodyTabKind::Method {
                    idx,
                    method,
                    address,
                    request,
                    stream,
                    options,
                } => {
                    let method = get_service(file_sets, &idx)?
                        .methods()
                        .nth(method)
                        .context("invalid method index")?;
                    Ok((
                        TabId::next(),
                        app::body::TabState::new_method(
                            method,
                            address,
                            JsonText::pretty(request),
                            stream,
                            options,
                        ),
                    ))
                }
                AppBodyTabKind::Options { idx } => {
                    let service = get_service(file_sets, &idx)?;

                    let options = services
                        .iter()
                        .find(|s| s.idx == idx)
                        .context("options tab has no associated service")?
                        .options
                        .clone();

                    Ok((
                        TabId::next(),
                        app::body::TabState::new_options(service, options),
                    ))
                }
            })
            .collect::<Result<BTreeMap<_, _>>>()?;

        let selected = self
            .selected
            .and_then(|selected| tabs.iter().nth(selected).map(|(&id, _)| id));

        Ok(app::body::State::new(tabs, selected))
    }
}

fn get_or_insert_file_set(
    vec: &mut Vec<prost_reflect::DescriptorPool>,
    files: &prost_reflect::DescriptorPool,
) -> Result<usize> {
    match vec.iter().position(|data| data == files) {
        Some(index) => Ok(index),
        None => {
            let index = vec.len();
            vec.push(files.clone());
            Ok(index)
        }
    }
}

fn get_service(
    vec: &[prost_reflect::DescriptorPool],
    idx: &AppServiceRef,
) -> Result<prost_reflect::ServiceDescriptor> {
    vec.get(idx.file_set)
        .context("invalid file set index")?
        .services()
        .nth(idx.service)
        .context("invalid service index")
}
