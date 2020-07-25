use std::sync::Arc;

use druid::Data;

use crate::address::Address;

#[derive(Clone, Debug, Data)]
pub enum ConnectionState {
    Uninit,
    Connected(Connection),
}

#[derive(Clone, Debug, Data)]
pub struct Connection {
    client: Arc<grpc::Client>,
}

impl ConnectionState {
    pub fn new() -> Self {
        ConnectionState::Uninit
    }

    pub fn connect(&mut self, address: &Address, tls: bool) -> grpc::Result<()> {
        let client_builder = grpc::ClientBuilder::new(&address.host, address.port);
        let client = if tls {
            client_builder.tls::<tls_api_rustls::TlsConnector>().build()
        } else {
            client_builder.build()
        }?;

        *self = ConnectionState::Connected(Connection {
            client: Arc::new(client),
        });
        Ok(())
    }
}

impl Default for ConnectionState {
    fn default() -> Self {
        ConnectionState::new()
    }
}
