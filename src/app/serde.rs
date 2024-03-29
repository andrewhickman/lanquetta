use std::{
    collections::BTreeMap,
    convert::{TryFrom, TryInto},
};

use anyhow::{Context, Error, Result};
use druid::piet::TextStorage;
use prost_reflect::{prost::Message, prost_types::FileDescriptorSet};
use prost_reflect::{DescriptorPool, DynamicMessage, ReflectMessage};
use serde::{
    de::{self, Deserializer},
    ser::{self, Serializer},
    Deserialize, Serialize,
};

use crate::{
    app,
    json::JsonText,
    widget::{TabId, TabsData},
};

use super::body::CompileOptions;

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
    file_descriptor_sets: Vec<DescriptorPoolSerde>,
    services: Vec<AppServiceState>,
    body: AppBodyState,
    compile_options: CompileOptions,
}

#[derive(Debug, Serialize, Deserialize, PartialEq)]
struct AppServiceRef {
    file_set: usize,
    service: String,
}

#[derive(Debug, Serialize, Deserialize)]
struct AppServiceState {
    #[serde(flatten)]
    idx: AppServiceRef,
    expanded: bool,
    options: app::sidebar::service::ServiceOptions,
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
        request_metadata: app::metadata::State,
        stream: app::body::StreamState,
        options: app::sidebar::service::ServiceOptions,
    },
    Options {
        #[serde(flatten)]
        idx: AppServiceRef,
    },
    Compile,
    Reflection {
        options: app::sidebar::service::ServiceOptions,
    },
}

#[derive(Debug)]
struct DescriptorPoolSerde(DescriptorPool);

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
                        service: service.service().full_name().to_owned(),
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
                                    service: method
                                        .method()
                                        .parent_service()
                                        .full_name()
                                        .to_owned(),
                                },
                                method: method.method().index(),
                                address: method.address().text().to_owned(),
                                request: method.request().text().as_str().to_owned(),
                                request_metadata: method.request().serde_metadata(),
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
                                    service: options.service().full_name().to_owned(),
                                },
                            }
                        }
                        app::body::TabState::Compile(_) => AppBodyTabKind::Compile,
                        app::body::TabState::Reflection(options) => AppBodyTabKind::Reflection {
                            options: options.service_options(),
                        },
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
            .map(DescriptorPoolSerde)
            .collect();

        Ok(AppState {
            file_descriptor_sets,
            services,
            body,
            compile_options: data.sidebar.compile_options().clone(),
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
            compile_options,
        } = self;

        let file_descriptor_sets: Vec<_> = file_descriptor_sets
            .into_iter()
            .map(|serde| serde.0)
            .collect();

        let service_states = services
            .iter()
            .map(|service| {
                Ok(app::sidebar::service::ServiceState::new(
                    get_service(&file_descriptor_sets, &service.idx)?,
                    service.expanded,
                    service.options.clone(),
                ))
            })
            .collect::<Result<Vec<_>>>()?;

        Ok(app::State {
            body: body.into_state(&file_descriptor_sets, &services, &compile_options)?,
            sidebar: app::sidebar::ServiceListState::new(service_states, compile_options),
            error: None,
        })
    }
}

impl AppBodyState {
    fn into_state(
        self,
        file_sets: &[prost_reflect::DescriptorPool],
        services: &[AppServiceState],
        compile_options: &CompileOptions,
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
                    request_metadata,
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
                            request_metadata,
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
                AppBodyTabKind::Compile => Ok((
                    TabId::next(),
                    app::body::TabState::new_compile(compile_options),
                )),
                AppBodyTabKind::Reflection { options } => {
                    Ok((TabId::next(), app::body::TabState::new_reflection(options)))
                }
            })
            .collect::<Result<BTreeMap<_, _>>>()?;

        let selected = self
            .selected
            .and_then(|selected| tabs.iter().nth(selected).map(|(&id, _)| id));

        Ok(app::body::State::new(tabs, selected))
    }
}

impl Serialize for DescriptorPoolSerde {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let bytes = self.0.encode_to_vec();
        let mut dynamic =
            DynamicMessage::decode(FileDescriptorSet::default().descriptor(), bytes.as_slice())
                .map_err(<S::Error as serde::ser::Error>::custom)?;

        for file in dynamic
            .get_field_by_name_mut("file")
            .unwrap()
            .as_list_mut()
            .unwrap()
        {
            // We don't use source code info and it bloats the config file.
            file.as_message_mut()
                .unwrap()
                .clear_field_by_name("source_code_info");
        }

        dynamic.serialize(serializer)
    }
}

impl<'de> Deserialize<'de> for DescriptorPoolSerde {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let dynamic =
            DynamicMessage::deserialize(FileDescriptorSet::default().descriptor(), deserializer)?;
        let bytes = dynamic.encode_to_vec();
        Ok(DescriptorPoolSerde(
            DescriptorPool::decode(bytes.as_slice())
                .map_err(<D::Error as serde::de::Error>::custom)?,
        ))
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
        .find(|s| s.full_name() == idx.service)
        .context("invalid service index")
}
