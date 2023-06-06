use std::{sync::Arc, time::SystemTime};

use anyhow::{Context, Error};
use dashmap::{mapref::entry::Entry, DashMap};
use futures::future::BoxFuture;
use http::{uri::Scheme, Uri};
use hyper::client::HttpConnector;
use hyper_rustls::HttpsConnectorBuilder;
use once_cell::sync::{Lazy, OnceCell};
use rustls::RootCertStore;
use tokio::sync::Mutex;
use tonic::transport::Channel;

use crate::grpc;

static CHANNELS: Lazy<DashMap<ChannelKey, Arc<Mutex<ChannelState>>>> = Lazy::new(Default::default);

#[derive(Clone, Hash, PartialEq, Eq)]
struct ChannelKey {
    uri: Uri,
    verify_certs: bool,
}

enum ChannelState {
    Pending(BoxFuture<'static, Result<Channel, grpc::Error>>),
    Ready(Channel),
    Error,
}

pub async fn get(uri: &Uri, verify_certs: bool) -> Result<Channel, grpc::Error> {
    let key = ChannelKey {
        uri: uri.clone(),
        verify_certs,
    };
    let state = match CHANNELS.entry(key) {
        Entry::Occupied(entry) => entry.get().clone(),
        Entry::Vacant(entry) => {
            let state = Arc::new(Mutex::new(ChannelState::new(uri.clone(), verify_certs)));
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
                *lock = ChannelState::new(uri.clone(), verify_certs);
            }
        }
    }
}

impl ChannelState {
    fn new(uri: Uri, verify_certs: bool) -> Self {
        ChannelState::Pending(Box::pin(connect(uri, verify_certs)))
    }
}

async fn connect(uri: Uri, verify_certs: bool) -> Result<Channel, grpc::Error> {
    let is_https = uri.scheme() == Some(&Scheme::HTTPS);
    let builder = Channel::builder(uri);

    if is_https {
        let mut http = HttpConnector::new();
        http.enforce_http(false);
        http.set_nodelay(true);

        let rustls_config = if verify_certs {
            static ROOT_STORE: OnceCell<RootCertStore> = OnceCell::new();

            let root_store = ROOT_STORE
                .get_or_try_init::<_, Error>(|| {
                    let mut roots = RootCertStore::empty();
                    for cert in rustls_native_certs::load_native_certs()? {
                        roots.add(&rustls::Certificate(cert.0))?;
                    }
                    Ok(roots)
                })
                .context("failed to load trusted root certificate store")?
                .clone();

            rustls::ClientConfig::builder()
                .with_safe_defaults()
                .with_root_certificates(root_store)
                .with_no_client_auth()
        } else {
            rustls::ClientConfig::builder()
                .with_safe_defaults()
                .with_custom_certificate_verifier(Arc::new(DangerousCertificateVerifier))
                .with_no_client_auth()
        };

        let https = HttpsConnectorBuilder::new()
            .with_tls_config(rustls_config)
            .https_only()
            .enable_http2()
            .wrap_connector(http);

        builder
            .connect_with_connector(https)
            .await
            .map_err(anyhow::Error::from)
            .map_err(grpc::Error::from)
    } else {
        builder
            .connect()
            .await
            .map_err(anyhow::Error::from)
            .map_err(grpc::Error::from)
    }
}

struct DangerousCertificateVerifier;

impl rustls::client::ServerCertVerifier for DangerousCertificateVerifier {
    fn verify_server_cert(
        &self,
        _end_entity: &rustls::Certificate,
        _intermediates: &[rustls::Certificate],
        _server_name: &rustls::ServerName,
        _scts: &mut dyn Iterator<Item = &[u8]>,
        _ocsp_response: &[u8],
        _now: SystemTime,
    ) -> Result<rustls::client::ServerCertVerified, rustls::Error> {
        Ok(rustls::client::ServerCertVerified::assertion())
    }

    fn verify_tls12_signature(
        &self,
        _message: &[u8],
        _cert: &rustls::Certificate,
        _dss: &rustls::DigitallySignedStruct,
    ) -> Result<rustls::client::HandshakeSignatureValid, rustls::Error> {
        Ok(rustls::client::HandshakeSignatureValid::assertion())
    }

    fn verify_tls13_signature(
        &self,
        _message: &[u8],
        _cert: &rustls::Certificate,
        _dss: &rustls::DigitallySignedStruct,
    ) -> Result<rustls::client::HandshakeSignatureValid, rustls::Error> {
        Ok(rustls::client::HandshakeSignatureValid::assertion())
    }
}
