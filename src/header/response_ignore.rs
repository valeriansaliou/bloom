// Bloom
//
// HTTP REST API caching middleware
// Copyright: 2017, Valerian Saliou <valerian@valeriansaliou.name>
// License: Mozilla Public License v2.0 (MPL v2.0)

pub const HEADER_NAME: &str = "bloom-response-ignore";

pub fn should_ignore(value: &str) -> bool {
    value.trim() == "1"
}
