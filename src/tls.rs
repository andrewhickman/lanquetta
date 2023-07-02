use std::{sync::Arc, time::SystemTime};

use anyhow::{Context, Error, Result};
use http::Uri;
use hyper::client::connect::Connection;
use hyper_rustls::{HttpsConnector, HttpsConnectorBuilder};
use once_cell::sync::OnceCell;
use rustls::RootCertStore;
use tokio::io::{AsyncRead, AsyncWrite};
use tower::{BoxError, Service};

pub fn wrap<C>(connector: C, verify_certs: bool) -> Result<HttpsConnector<C>>
where
    C: Service<Uri>,
    C::Response: Connection + AsyncRead + AsyncWrite,
    C::Future: Send + 'static,
    C::Error: Into<BoxError>,
{
    let rustls_config = if verify_certs {
        rustls::ClientConfig::builder()
            .with_safe_defaults()
            .with_root_certificates(native_root_cert_store()?)
            .with_no_client_auth()
    } else {
        rustls::ClientConfig::builder()
            .with_safe_defaults()
            .with_custom_certificate_verifier(Arc::new(DangerousCertificateVerifier))
            .with_no_client_auth()
    };

    Ok(HttpsConnectorBuilder::new()
        .with_tls_config(rustls_config)
        .https_only()
        .enable_http2()
        .wrap_connector(connector))
}

fn native_root_cert_store() -> Result<RootCertStore> {
    static ROOT_STORE: OnceCell<RootCertStore> = OnceCell::new();

    Ok(ROOT_STORE
        .get_or_try_init::<_, Error>(|| {
            let mut roots = RootCertStore::empty();
            for cert in rustls_native_certs::load_native_certs()? {
                roots.add(&rustls::Certificate(cert.0))?;
            }
            Ok(roots)
        })
        .context("failed to load trusted root certificate store")?
        .clone())
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
