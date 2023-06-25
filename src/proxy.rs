// TODO exclusions
// On all platforms: parse HTTP_PROXY, HTTPS_PROXY, NO_PROXY
// See https://about.gitlab.com/blog/2021/01/27/we-need-to-talk-no-proxy/

// On windows: fallback to registry
// https://github.com/seanmonstar/reqwest/issues/1444
// https://github.com/seanmonstar/reqwest/blob/e02df1f448d845fe01e6ea82c76ec89a59e5d568/src/proxy.rs#L898C4-L898C30

// TODO: reloading values from env

use anyhow::Result;
use druid::Data;
use http::Uri;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, Data)]
#[serde(from = "ProxyKind", into = "ProxyKind")]
pub struct Proxy {}

#[derive(Serialize, Deserialize)]
enum ProxyKind {
    None,
    System,
    #[serde(with = "http_serde::uri")]
    Custom(Uri),
}

impl Proxy {
    pub fn none() -> Self {
        todo!()
    }

    pub fn system() -> Result<Self> {
        todo!()
    }

    pub fn custom(uri: Uri) -> Self {
        todo!()
    }

    pub fn get_proxy(&self, target: &Uri) -> Option<Uri> {
        todo!()
    }

    pub fn get_default(&self) -> Option<Uri> {
        todo!()
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
        todo!()
    }
}
