use std::{convert::TryFrom, convert::TryInto, sync::Arc};

use anyhow::{Context, Error, Result};
use protobuf::descriptor::FileDescriptorSet;
use serde::{
    de::{self, Deserializer},
    ser::{self, Serializer},
    Deserialize, Serialize,
};

use crate::{app, protobuf::ProtobufService};

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
}

#[derive(Debug, Serialize, Deserialize)]
struct AppServiceState {
    fd_set: usize,
    service: usize,
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
                    get_or_insert_fd_set(&mut file_descriptor_sets, service.service().raw_files())?;
                Ok(AppServiceState {
                    fd_set,
                    service: service.service().raw_files_index(),
                    expanded: service.expanded(),
                })
            })
            .collect::<Result<Vec<_>>>()?;

        Ok(AppState {
            file_descriptor_sets,
            services,
        })
    }
}

impl TryInto<app::State> for AppState {
    type Error = Error;

    fn try_into(self) -> Result<app::State, Self::Error> {
        let AppState {
            file_descriptor_sets,
            services,
        } = self;

        Ok(app::State {
            sidebar: services
                .into_iter()
                .map(|service| {
                    Ok(app::sidebar::ServiceState::new(
                        file_descriptor_sets
                            .get(service.fd_set)
                            .context("invalid fd set index")?
                            .services
                            .get(service.service)
                            .context("invalid service index")?
                            .clone(),
                        service.expanded,
                    ))
                })
                .collect::<Result<app::sidebar::ServiceListState>>()?,
            body: Default::default(),
        })
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
    match vec.binary_search_by_key(&Arc::as_ptr(&files), |data| Arc::as_ptr(&data.files)) {
        Ok(index) => Ok(index),
        Err(index) => {
            vec.insert(index, AppFileDescriptorSetState::try_from(files)?);
            Ok(index)
        }
    }
}
