// Bloom
//
// HTTP REST API caching middleware
// Copyright: 2017, Valerian Saliou <valerian@valeriansaliou.name>
// License: Mozilla Public License v2.0 (MPL v2.0)

pub struct HeaderResponseBuilder;
pub struct HeaderResponse {
    bucket: String,
    ttl: u32,
    ignore: bool
}

impl HeaderResponseBuilder {
    pub fn new(bucket: String, ttl: u32, ignore: bool) -> HeaderResponse {
        HeaderResponse {
            bucket: bucket,
            ttl: ttl,
            ignore: ignore
        }
    }
}

// TODO: extend hyper typed header traits?
