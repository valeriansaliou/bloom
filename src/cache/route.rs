// Bloom
//
// HTTP REST API caching middleware
// Copyright: 2017, Valerian Saliou <valerian@valeriansaliou.name>
// License: Mozilla Public License v2.0 (MPL v2.0)

use hyper::{Method, StatusCode, HttpVersion};
use farmhash;

pub struct CacheRoute;

pub const ROUTE_HASH_SIZE: usize = 8;

impl CacheRoute {
    // TODO: adjust hash system to concat. authorization hash + route hash
    // this makes hash collisions impossible and thus cache leak across != \
    //   authorization values not possible.
    // ie new key format: bloom:{shard}:{auth_h<HASH(auth or nil)>}:{ns_h}

    pub fn gen_ns_from_hash(shard: u8, namespace_hash: &str) -> String {
        format!("bloom:{}:{}", shard, namespace_hash)
    }

    pub fn gen_ns(shard: u8, version: HttpVersion, method: &Method, path: &str,
                    query: Option<&str>, authorization: &str) -> String {
        let namespace_raw = format!("[{}][{}][{}][{}][{}]", version, method,
            path, query.unwrap_or(""), authorization);
        let namespace_hash = Self::hash(&namespace_raw);

        debug!("Generated namespace: {} with hash: {}", namespace_raw,
            namespace_hash);

        Self::gen_ns_from_hash(shard, namespace_hash.as_str())
    }

    pub fn hash(hash: &str) -> String {
        format!("{:x}", farmhash::fingerprint32(hash.as_bytes()))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn it_generates_valid_ns() {
        assert_eq!(CacheRoute::gen_ns(
            0, HttpVersion::Http11, &Method::Get, "/", Some(""), ""),
            "bloom:0:d180fb05", "[shard=0][auth=no] HTTP/1.1 GET /");
        assert_eq!(CacheRoute::gen_ns(
            0, HttpVersion::Http11, &Method::Post, "/login", Some(""), ""),
            "bloom:0:27859e5b", "[shard=0][auth=no] HTTP/1.1 POST /login");
        assert_eq!(CacheRoute::gen_ns(
            7, HttpVersion::Http11, &Method::Options, "/feed", Some(""), "8ab"),
            "bloom:7:263a7d5", "[shard=7][auth=yes] HTTP/1.1 OPTIONS /feed");
        assert_eq!(CacheRoute::gen_ns(
            80, HttpVersion::H2, &Method::Head, "/user", Some("u=1"), "2d"),
            "bloom:80:660a39b2", "[shard=80][auth=yes] h2 HEAD /feed");
        assert_eq!(ROUTE_HASH_SIZE, CacheRoute::hash("7gCq81kzO5").len(),
            "Route size should be 8 (dynamic)");
    }
}

