use http::Uri;
use tokio::sync::broadcast;

use crate::grpc;

pub enum ClientState {
    NotConnected,
    ConnectFailed,
    ConnectInProgress {
        uri: Uri,
        sender: broadcast::Sender<grpc::ConnectResult>,
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

    /// Get a channel that will return a client for the given uri. The channel may return `RecvError::Closed`
    /// if `get` and `set` are called with a different uri before the connection completes.
    pub fn get(&mut self, uri: &Uri) -> broadcast::Receiver<grpc::ConnectResult> {
        match self {
            ClientState::Connected {
                uri: prev_uri,
                client,
            } if prev_uri == uri => {
                let (sender, receiver) = broadcast::channel(1);
                sender.send(Ok(client.clone())).unwrap();
                receiver
            }
            ClientState::ConnectInProgress {
                uri: prev_uri,
                sender,
            } if prev_uri == uri => sender.subscribe(),
            _ => {
                let (sender, receiver) = broadcast::channel(1);
                tokio::spawn({
                    let uri = uri.clone();
                    let sender = sender.clone();
                    async move {
                        let client = grpc::Client::new(uri).await;
                        let _ = sender.send(client);
                    }
                });
                *self = ClientState::ConnectInProgress {
                    uri: uri.clone(),
                    sender,
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
