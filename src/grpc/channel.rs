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
    Error,
}

pub async fn get(uri: &Uri) -> Result<Channel, grpc::Error> {
    let state = match CHANNELS.entry(uri.clone()) {
        Entry::Occupied(entry) => entry.get().clone(),
        Entry::Vacant(entry) => {
            let state = Arc::new(Mutex::new(ChannelState::new(uri.clone())));
            entry.insert(Arc::clone(&state));
            state
        }
    };

    loop {
        let mut lock = state.lock().await;
        match &mut *lock {
            ChannelState::Pending(fut) => {
                let result = fut.await;
                *lock = match &result {
                    Ok(channel) => ChannelState::Ready(channel.clone()),
                    Err(err) => {
                        tracing::error!("failed to connect to {}: {:?}", uri, err);
                        ChannelState::Error
                    }
                };
                return result;
            }
            ChannelState::Ready(channel) => return Ok(channel.clone()),
            ChannelState::Error => {
                *lock = ChannelState::new(uri.clone());
            }
        }
    }
}

impl ChannelState {
    fn new(uri: Uri) -> Self {
        ChannelState::Pending(Box::pin(connect(uri)))
    }
}

async fn connect(uri: Uri) -> Result<Channel, grpc::Error> {
    Channel::builder(uri)
        .connect()
        .await
        .map_err(anyhow::Error::from)
        .map_err(grpc::Error::from)
}
