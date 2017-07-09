// Bloom
//
// HTTP REST API caching middleware
// Copyright: 2017, Valerian Saliou <valerian@valeriansaliou.name>
// License: Mozilla Public License v2.0 (MPL v2.0)

use std::str::from_utf8;

use hyper::header::Headers;

use super::defaults;
use header::request_shard::HeaderRequestBloomRequestShard;

pub struct ProxyHeader;

impl ProxyHeader {
    pub fn parse_from_request(headers: &Headers) -> (&str, u8) {
        // Request header: 'Authorization'
        let auth = match headers.get_raw("authorization") {
            None => defaults::REQUEST_AUTHORIZATION_DEFAULT,
            Some(value) => from_utf8(value.one().unwrap_or(&[])).unwrap_or(
                defaults::REQUEST_AUTHORIZATION_DEFAULT)
        };

        // Request header: 'Bloom-Request-Shard'
        let shard = match headers.get::<HeaderRequestBloomRequestShard>() {
            None => defaults::REQUEST_SHARD_DEFAULT,
            Some(value) => value.0
        };

        (auth, shard)
    }
}
