// Bloom
//
// HTTP REST API caching middleware
// Copyright: 2017, Valerian Saliou <valerian@valeriansaliou.name>
// License: Mozilla Public License v2.0 (MPL v2.0)

pub const HEADER_NAME: &str = "bloom-response-ttl";

pub fn parse_ttl(value: &str) -> Option<usize> {
    value.trim().parse().ok()
}
