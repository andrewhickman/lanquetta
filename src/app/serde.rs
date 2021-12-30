use std::{
    collections::BTreeMap,
    convert::{TryFrom, TryInto},
};

use anyhow::{Context, Error, Result};
use druid::piet::TextStorage;
use prost::Message;
use prost_reflect::FileDescriptor;
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

#[derive(Debug, Serialize, Deserialize)]
struct AppServiceRef {
    file_set: usize,
    service: usize,
}

#[derive(Debug, Serialize, Deserialize)]
struct AppServiceState {
    #[serde(flatten)]
    idx: AppServiceRef,
    expanded: bool,
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
                let file_set = get_or_insert_file_set(
                    &mut file_descriptor_sets,
                    service.service().parent_file(),
                )?;
                Ok(AppServiceState {
                    idx: AppServiceRef {
                        file_set,
                        service: service.service().index(),
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
                    let file_set = get_or_insert_file_set(
                        &mut file_descriptor_sets,
                        tab.method().parent_file(),
                    )?;
                    Ok(AppBodyTabState {
                        idx: AppServiceRef {
                            file_set,
                            service: tab.method().parent_service().index(),
                        },
                        method: tab.method().index(),
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

        let file_descriptor_sets = file_descriptor_sets
            .into_iter()
            .map(|f| {
                let bytes = f.file_descriptor_set().encode_to_vec();
                base64::encode(bytes)
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
                let bytes = base64::decode(b64)?;
                anyhow::Ok(FileDescriptor::decode(bytes.as_ref())?)
            })
            .collect::<Result<Vec<_>, _>>()?;

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
            body: body.into_state(&file_descriptor_sets)?,
            error: None,
        })
    }
}

impl AppBodyState {
    fn into_state(self, file_sets: &[prost_reflect::FileDescriptor]) -> Result<app::body::State> {
        let tabs = self
            .tabs
            .into_iter()
            .map(|tab| {
                let method = get_service(file_sets, &tab.idx)?
                    .methods()
                    .nth(tab.method)
                    .context("invalid method index")?;
                Ok((
                    TabId::next(),
                    app::body::TabState::new(
                        method,
                        tab.address,
                        JsonText::pretty(tab.request),
                        tab.stream,
                    ),
                ))
            })
            .collect::<Result<BTreeMap<_, _>>>()?;

        let selected = self
            .selected
            .and_then(|selected| tabs.iter().nth(selected).map(|(&id, _)| id));

        Ok(app::body::State::new(tabs, selected))
    }
}

fn get_or_insert_file_set(
    vec: &mut Vec<prost_reflect::FileDescriptor>,
    files: &prost_reflect::FileDescriptor,
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
    vec: &[prost_reflect::FileDescriptor],
    idx: &AppServiceRef,
) -> Result<prost_reflect::ServiceDescriptor> {
    vec.get(idx.file_set)
        .context("invalid file set index")?
        .services()
        .nth(idx.service)
        .context("invalid service index")
}
