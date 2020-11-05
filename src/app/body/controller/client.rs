use http::Uri;

use crate::{grpc, oneshot};

pub enum ClientState {
    NotConnected,
    ConnectFailed,
    ConnectInProgress {
        uri: Uri,
        receiver: oneshot::Receiver<grpc::ConnectResult>,
    },
    Connected {
        uri: Uri,
        client: grpc::Client,
    },
}

impl ClientState {
    pub fn new() -> Self {
        ClientState::NotConnected
    }

    /// Get a channel that will return a client for the given uri.
    pub fn get(&mut self, uri: &Uri) -> oneshot::Receiver<grpc::ConnectResult> {
        match self {
            ClientState::Connected {
                uri: prev_uri,
                client,
            } if prev_uri == uri => {
                let client = client.clone();
                oneshot::new(async move { Ok(client) })
            }
            ClientState::ConnectInProgress {
                uri: prev_uri,
                receiver,
            } if prev_uri == uri => receiver.clone(),
            _ => {
                let receiver = oneshot::new(grpc::Client::new(uri.clone()));
                *self = ClientState::ConnectInProgress {
                    uri: uri.clone(),
                    receiver: receiver.clone(),
                };
                receiver
            }
        }
    }

    /// Set the result of a channel returned by `get`. If `get` was called with a different uri afer the initial call,
    /// this has no effect.
    pub fn set(&mut self, uri: &Uri, result: grpc::ConnectResult) {
        match self {
            ClientState::ConnectInProgress { uri: prev_uri, .. } if prev_uri == uri => {
                *self = match result {
                    Ok(client) => ClientState::Connected {
                        uri: uri.clone(),
                        client,
                    },
                    Err(err) => {
                        log::info!("Connection to {} failed: {:?}", uri, err);
                        ClientState::ConnectFailed
                    }
                }
            }
            _ => (),
        }
    }

    pub fn reset(&mut self) {
        *self = ClientState::NotConnected
    }
}
