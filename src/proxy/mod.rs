// TODO exclusions
// On all platforms: parse HTTP_PROXY, HTTPS_PROXY, NO_PROXY
// See https://about.gitlab.com/blog/2021/01/27/we-need-to-talk-no-proxy/

// On windows: fallback to registry
// https://github.com/seanmonstar/reqwest/issues/1444
// https://github.com/seanmonstar/reqwest/blob/e02df1f448d845fe01e6ea82c76ec89a59e5d568/src/proxy.rs#L898C4-L898C30

// TODO: reloading values from env

// TODO proxy auth
// TODO proxy tls validation

mod ignore;
mod sys;

use std::sync::Arc;

use anyhow::Result;
use druid::Data;
use http::Uri;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, Data)]
#[serde(from = "ProxyKind", into = "ProxyKind")]
pub struct Proxy {
    inner: Option<Arc<ProxyInner>>,
}

#[derive(Debug)]
struct ProxyInner {}

#[derive(Debug, Serialize, Deserialize)]
pub enum ProxyKind {
    None,
    System,
    #[serde(with = "http_serde::uri")]
    Custom(Uri),
}

impl Proxy {
    pub fn none() -> Self {
        Proxy { inner: None }
    }

    pub fn system() -> Result<Self> {
        todo!()
    }

    pub fn custom(_: Uri) -> Self {
        todo!()
    }

    pub fn kind(&self) -> ProxyKind {
        ProxyKind::None
    }

    pub fn get_proxy(&self, _: &Uri) -> Option<Uri> {
        None
    }

    pub fn get_default(&self) -> Option<Uri> {
        None
    }

    pub fn verify_certs(&self) -> bool {
        false
    }

    pub fn auth(&self) -> String {
        String::default()
    }
}

impl From<ProxyKind> for Proxy {
    fn from(kind: ProxyKind) -> Self {
        match kind {
            ProxyKind::None => Proxy::none(),
            ProxyKind::System => Proxy::system().unwrap_or_else(|err| {
                tracing::error!("failed to load system proxy: {:?}", err);
                Proxy::none()
            }),
            ProxyKind::Custom(uri) => Proxy::custom(uri),
        }
    }
}

impl From<Proxy> for ProxyKind {
    fn from(proxy: Proxy) -> Self {
        proxy.kind()
    }
}
