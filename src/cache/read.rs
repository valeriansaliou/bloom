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

        // TODO: CacheStore::get()
            // TODO: return future w/ Ok if valid response
            // TODO: return future w/ Err if no response (or expired?)

        // TODO: IMPORTANT: if memcached is down, fallback to DIRECT in any \
        //   case, but throw an error log
    }
}
