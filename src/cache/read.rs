// Bloom
//
// HTTP REST API caching middleware
// Copyright: 2017, Valerian Saliou <valerian@valeriansaliou.name>
// License: Mozilla Public License v2.0 (MPL v2.0)

use hyper::{HttpVersion, Method};

pub struct CacheRead;

impl CacheRead {
    pub fn acquire(ns: &str) {
        // TODO: Not implemented
    }
}
