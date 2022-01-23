use std::sync::Arc;

use dashmap::{mapref::entry::Entry, DashMap};
use futures::future::BoxFuture;
use http::{uri::Scheme, Uri};
use once_cell::sync::Lazy;
use tokio::sync::Mutex;
use tonic::transport::{Channel, ClientTlsConfig};

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

const ALPN_H2: &[u8] = b"h2";

async fn connect(uri: Uri, verify_certs: bool) -> Result<Channel, grpc::Error> {
    let is_https = uri.scheme() == Some(&Scheme::HTTPS);
    let mut builder = Channel::builder(uri);

    if is_https {
        let mut tls_config = ClientTlsConfig::new();
        if !verify_certs {
            let mut rustls_config = rustls::ClientConfig::new();
            rustls_config.set_protocols(&[ALPN_H2.to_vec()]);
            rustls_config
                .dangerous()
                .set_certificate_verifier(Arc::new(DangerousCertificateVerifier));
            tls_config = tls_config.rustls_client_config(rustls_config);
        }

        builder = builder
            .tls_config(tls_config)
            .map_err(anyhow::Error::from)
            .map_err(grpc::Error::from)?;
    }

    builder
        .connect()
        .await
        .map_err(anyhow::Error::from)
        .map_err(grpc::Error::from)
}

struct DangerousCertificateVerifier;

impl rustls::ServerCertVerifier for DangerousCertificateVerifier {
    fn verify_server_cert(
        &self,
        _roots: &rustls::RootCertStore,
        _presented_certs: &[rustls::Certificate],
        _dns_name: webpki::DNSNameRef,
        _ocsp_response: &[u8],
    ) -> Result<rustls::ServerCertVerified, rustls::TLSError> {
        Ok(rustls::ServerCertVerified::assertion())
    }

    fn verify_tls12_signature(
        &self,
        _message: &[u8],
        _cert: &rustls::Certificate,
        _dss: &rustls::internal::msgs::handshake::DigitallySignedStruct,
    ) -> Result<rustls::HandshakeSignatureValid, rustls::TLSError> {
        Ok(rustls::HandshakeSignatureValid::assertion())
    }

    fn verify_tls13_signature(
        &self,
        _message: &[u8],
        _cert: &rustls::Certificate,
        _dss: &rustls::internal::msgs::handshake::DigitallySignedStruct,
    ) -> Result<rustls::HandshakeSignatureValid, rustls::TLSError> {
        Ok(rustls::HandshakeSignatureValid::assertion())
    }
}
