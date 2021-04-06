use std::sync::Arc;

use dashmap::{mapref::entry::Entry, DashMap};
use futures::future::BoxFuture;
use http::Uri;
use once_cell::sync::Lazy;
use tokio::sync::Mutex;
use tonic::transport::Channel;

use crate::grpc;

static CHANNELS: Lazy<DashMap<Uri, Arc<Mutex<ChannelState>>>> = Lazy::new(Default::default);

enum ChannelState {
    Pending(BoxFuture<'static, Result<Channel, grpc::Error>>),
    Ready(Channel),
    Error(grpc::Error),
}

pub async fn get(uri: Uri) -> Result<Channel, grpc::Error> {
    let state = match CHANNELS.entry(uri) {
        Entry::Occupied(entry) => entry.get().clone(),
        Entry::Vacant(entry) => {
            let state = ChannelState::new(entry.key().clone());
            entry.insert(Arc::clone(&state));
            state
        }
    };

    let mut lock = state.lock().await;
    match &mut *lock {
        ChannelState::Pending(fut) => {
            let result = fut.await;
            *lock = match &result {
                Ok(channel) => ChannelState::Ready(channel.clone()),
                Err(error) => ChannelState::Error(error.clone()),
            };
            result
        }
        ChannelState::Ready(channel) => Ok(channel.clone()),
        ChannelState::Error(error) => Err(error.clone()),
    }
}

impl ChannelState {
    fn new(uri: Uri) -> Arc<Mutex<Self>> {
        Arc::new(Mutex::new(ChannelState::Pending(Box::pin(connect(uri)))))
    }
}

async fn connect(uri: Uri) -> Result<Channel, grpc::Error> {
    Channel::builder(uri)
        .connect()
        .await
        .map_err(anyhow::Error::from)
        .map_err(grpc::Error::from)
}
