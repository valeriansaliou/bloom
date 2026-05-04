// Bloom
//
// HTTP REST API caching middleware
// Copyright: 2017, Valerian Saliou <valerian@valeriansaliou.name>
// License: Mozilla Public License v2.0 (MPL v2.0)

pub const HEADER_NAME: &str = "bloom-request-shard";

pub fn parse_shard(value: &str) -> Option<u8> {
    value.trim().parse().ok()
}
