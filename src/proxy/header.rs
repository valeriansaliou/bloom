// Bloom
//
// HTTP REST API caching middleware
// Copyright: 2017, Valerian Saliou <valerian@valeriansaliou.name>
// License: Mozilla Public License v2.0 (MPL v2.0)

use hyper::header::{ETag, Header, Vary};
use hyper::Headers;
use std::str::from_utf8;
use unicase::Ascii;

use super::defaults;
use crate::{header::request_shard::HeaderRequestBloomRequestShard, APP_CONF};

pub struct ProxyHeader;

impl ProxyHeader {
    pub fn parse_from_request(headers: Headers) -> (Headers, String, u8) {
        // Request header: 'Authorization'
        let auth = headers
            .get_raw("authorization")
            .map_or(defaults::REQUEST_AUTHORIZATION_DEFAULT, |value| {
                from_utf8(value.one().unwrap_or(&[]))
                    .unwrap_or(defaults::REQUEST_AUTHORIZATION_DEFAULT)
            })
            .to_string();

        // Request header: 'Bloom-Request-Shard'
        let shard = headers
            .get::<HeaderRequestBloomRequestShard>()
            .map_or_else(|| APP_CONF.proxy.shard_default, |value| value.0);

        (headers, auth, shard)
    }

    pub fn set_etag(headers: &mut Headers, etag: ETag) {
        headers.set::<Vary>(Vary::Items(vec![Ascii::new(
            ETag::header_name().to_string(),
        )]));

        headers.set::<ETag>(etag);
    }
}
