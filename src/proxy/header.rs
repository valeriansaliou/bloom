// Bloom
//
// HTTP REST API caching middleware
// Copyright: 2017, Valerian Saliou <valerian@valeriansaliou.name>
// License: Mozilla Public License v2.0 (MPL v2.0)

use http::header::{HeaderMap, HeaderValue, AUTHORIZATION, ETAG, VARY};

use super::defaults;
use crate::header::request_shard;
use crate::APP_CONF;

pub struct ProxyHeader;

impl ProxyHeader {
    pub fn parse_from_request(headers: &HeaderMap) -> (String, u8) {
        let auth = headers
            .get(AUTHORIZATION)
            .and_then(|v| v.to_str().ok())
            .unwrap_or(defaults::REQUEST_AUTHORIZATION_DEFAULT)
            .to_string();

        let shard = headers
            .get(request_shard::HEADER_NAME)
            .and_then(|v| v.to_str().ok())
            .and_then(|v| request_shard::parse_shard(v))
            .unwrap_or(APP_CONF.proxy.shard_default);

        (auth, shard)
    }

    pub fn set_etag(headers: &mut HeaderMap, etag: &str) {
        headers.insert(VARY, HeaderValue::from_static("etag"));

        if let Ok(value) = HeaderValue::from_str(&format!("\"{}\"", etag)) {
            headers.insert(ETAG, value);
        }
    }
}
