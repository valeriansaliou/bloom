// Bloom
//
// HTTP REST API caching middleware
// Copyright: 2017, Valerian Saliou <valerian@valeriansaliou.name>
// License: Mozilla Public License v2.0 (MPL v2.0)

extern crate hyper;

use self::hyper::{Method, StatusCode};

pub struct Read;

impl Read {
    pub fn gen_ns(method: Method, path: &str, authorization: &str) -> String {
        // TODO: Not implemented

        // returns ns
        return String::new()
    }

    pub fn acquire(ns: &str) {
        // TODO: Not implemented
    }
}
