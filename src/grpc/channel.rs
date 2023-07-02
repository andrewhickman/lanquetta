use std::sync::Arc;

use anyhow::Result;
use dashmap::{mapref::entry::Entry, DashMap};
use futures::future::BoxFuture;
use http::{uri::Scheme, Uri};
use hyper::client::HttpConnector;
use once_cell::sync::Lazy;
use tokio::sync::Mutex;
use tonic::transport::Channel;

use crate::{proxy::Proxy, tls};

static CHANNELS: Lazy<DashMap<ChannelKey, Arc<Mutex<ChannelState>>>> = Lazy::new(Default::default);

#[derive(Clone, Hash, PartialEq, Eq)]
struct ChannelKey {
    uri: Uri,
    verify_certs: bool,
}

enum ChannelState {
    Pending(BoxFuture<'static, Result<Channel>>),
    Ready(Channel),
    Error,
}

pub async fn get(uri: &Uri, verify_certs: bool, proxy: Proxy) -> Result<Channel> {
    let key = ChannelKey {
        uri: uri.clone(),
        verify_certs,
    };
    let state = match CHANNELS.entry(key) {
        Entry::Occupied(entry) => entry.get().clone(),
        Entry::Vacant(entry) => {
            let state = Arc::new(Mutex::new(ChannelState::new(
                uri.clone(),
                verify_certs,
                proxy.clone(),
            )));
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
                *lock = ChannelState::new(uri.clone(), verify_certs, proxy.clone());
            }
        }
    }
}

impl ChannelState {
    fn new(uri: Uri, verify_certs: bool, proxy: Proxy) -> Self {
        ChannelState::Pending(Box::pin(connect(uri, verify_certs, proxy)))
    }
}

async fn connect(uri: Uri, verify_certs: bool, proxy: Proxy) -> Result<Channel> {
    let is_https = uri.scheme() == Some(&Scheme::HTTPS);
    let builder = Channel::builder(uri);

    if is_https {
        let mut http = HttpConnector::new();
        http.enforce_http(false);
        http.set_nodelay(true);

        let https = tls::wrap(http, verify_certs)?;

        Ok(builder.connect_with_connector(https).await?)
    } else {
        Ok(builder.connect().await?)
    }
}
