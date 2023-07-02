// TODO exclusions
// On all platforms: parse HTTP_PROXY, HTTPS_PROXY, NO_PROXY
// See https://about.gitlab.com/blog/2021/01/27/we-need-to-talk-no-proxy/

// On windows: fallback to registry
// https://github.com/seanmonstar/reqwest/issues/1444
// https://github.com/seanmonstar/reqwest/blob/e02df1f448d845fe01e6ea82c76ec89a59e5d568/src/proxy.rs#L898C4-L898C30

// TODO: reloading values from env

// TODO proxy auth
// TODO proxy tls validation

use druid::Data;
use http::Uri;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, Data)]
#[serde(tag = "kind")]
pub enum Proxy {
    None,
    Custom {
        #[serde(with = "http_serde::uri")]
        #[data(same_fn = "PartialEq::eq")]
        uri: Uri,
        verify_certs: bool,
    },
}
