// Bloom
//
// HTTP REST API caching middleware
// Copyright: 2017, Valerian Saliou <valerian@valeriansaliou.name>
// License: Mozilla Public License v2.0 (MPL v2.0)

pub struct HeaderStrategyBuilder;
pub struct HeaderStrategy {
    bucket: String,
    ttl: u32,
    ignore: bool
}

impl HeaderStrategyBuilder {
    pub fn new(bucket: String, ttl: u32, ignore: bool) -> HeaderStrategy {
        HeaderStrategy {
            bucket: bucket,
            ttl: ttl,
            ignore: ignore
        }
    }
}
