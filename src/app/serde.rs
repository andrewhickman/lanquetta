use std::{
    collections::BTreeMap,
    convert::{TryFrom, TryInto},
    sync::Arc,
};

use anyhow::{Context, Error, Result};
use druid::piet::TextStorage;
use protobuf::descriptor::FileDescriptorSet;
use serde::{
    de::{self, Deserializer},
    ser::{self, Serializer},
    Deserialize, Serialize,
};

use crate::{
    app,
    protobuf::ProtobufService,
    widget::{TabId, TabsData},
};

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
    file_descriptor_sets: Vec<AppFileDescriptorSetState>,
    services: Vec<AppServiceState>,
    body: AppBodyState,
}

#[derive(Debug, Serialize, Deserialize)]
struct AppServiceRef {
    fd_set: usize,
    service: usize,
}

#[derive(Debug, Serialize, Deserialize)]
struct AppServiceState {
    #[serde(flatten)]
    idx: AppServiceRef,
    expanded: bool,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(try_from = "Arc<FileDescriptorSet>")]
struct AppFileDescriptorSetState {
    #[serde(flatten)]
    files: Arc<FileDescriptorSet>,
    #[serde(skip)]
    services: Vec<ProtobufService>,
}

#[derive(Debug, Serialize, Deserialize)]
struct AppBodyState {
    tabs: Vec<AppBodyTabState>,
    selected: Option<usize>,
}

#[derive(Debug, Serialize, Deserialize)]
struct AppBodyTabState {
    #[serde(flatten)]
    idx: AppServiceRef,
    method: usize,
    address: String,
    request: String,
    stream: app::body::stream::State,
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
                let fd_set =
                    get_or_insert_fd_set(&mut file_descriptor_sets, service.service().fd_set())?;
                Ok(AppServiceState {
                    idx: AppServiceRef {
                        fd_set,
                        service: service.service().service_index(),
                    },
                    expanded: service.expanded(),
                })
            })
            .collect::<Result<Vec<_>>>()?;

        let body = AppBodyState {
            tabs: data
                .body
                .tabs()
                .map(|(_, tab)| {
                    let fd_set =
                        get_or_insert_fd_set(&mut file_descriptor_sets, tab.method().fd_set())?;
                    Ok(AppBodyTabState {
                        idx: AppServiceRef {
                            fd_set,
                            service: tab.method().service_index(),
                        },
                        method: tab.method().method_index(),
                        address: tab.address().text().to_owned(),
                        request: tab.request().text().as_str().to_owned(),
                        stream: tab.stream().clone(),
                    })
                })
                .collect::<Result<Vec<_>>>()?,
            selected: data
                .body
                .selected()
                .and_then(|selected| data.body.tabs().position(|(id, _)| id == selected)),
        };

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

        Ok(app::State {
            sidebar: services
                .into_iter()
                .map(|service| {
                    Ok(app::sidebar::service::ServiceState::new(
                        get_service(&file_descriptor_sets, &service.idx)?,
                        service.expanded,
                    ))
                })
                .collect::<Result<app::sidebar::ServiceListState>>()?,
            body: body.to_state(&file_descriptor_sets)?,
            error: None,
        })
    }
}

impl AppBodyState {
    fn to_state(self, fd_sets: &[AppFileDescriptorSetState]) -> Result<app::body::State> {
        let tabs = self
            .tabs
            .into_iter()
            .map(|tab| {
                let method = get_service(fd_sets, &tab.idx)?
                    .get_method(tab.method)
                    .context("invalid method index")?
                    .clone();
                Ok((
                    TabId::next(),
                    app::body::TabState::new(method, tab.address, tab.request, tab.stream),
                ))
            })
            .collect::<Result<BTreeMap<_, _>>>()?;

        let selected = self
            .selected
            .and_then(|selected| tabs.iter().nth(selected).map(|(&id, _)| id));

        Ok(app::body::State::new(tabs, selected))
    }
}

impl TryFrom<Arc<FileDescriptorSet>> for AppFileDescriptorSetState {
    type Error = Error;

    fn try_from(files: Arc<FileDescriptorSet>) -> Result<Self, Self::Error> {
        Ok(AppFileDescriptorSetState {
            services: ProtobufService::load(&files)?,
            files,
        })
    }
}

fn get_or_insert_fd_set(
    vec: &mut Vec<AppFileDescriptorSetState>,
    files: Arc<FileDescriptorSet>,
) -> Result<usize> {
    match vec.iter().position(|data| Arc::ptr_eq(&data.files, &files)) {
        Some(index) => Ok(index),
        None => {
            let index = vec.len();
            vec.push(AppFileDescriptorSetState::try_from(files)?);
            Ok(index)
        }
    }
}

fn get_service(vec: &[AppFileDescriptorSetState], idx: &AppServiceRef) -> Result<ProtobufService> {
    Ok(vec
        .get(idx.fd_set)
        .context("invalid fd set index")?
        .services
        .get(idx.service)
        .context("invalid service index")?
        .clone())
}
