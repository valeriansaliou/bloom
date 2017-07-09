// Bloom
//
// HTTP REST API caching middleware
// Copyright: 2017, Valerian Saliou <valerian@valeriansaliou.name>
// License: Mozilla Public License v2.0 (MPL v2.0)

use hyper::{Method, HttpVersion};
use farmhash;

pub struct CacheRoute;

pub const ROUTE_HASH_SIZE: usize = 8;

impl CacheRoute {
    pub fn gen_ns_from_hash(shard: u8, auth_hash: &str, bucket_hash: &str) ->
        String {
        format!("bloom:{}:{}:{}", shard, auth_hash, bucket_hash)
    }

    pub fn gen_ns(shard: u8, authorization: &str, version: HttpVersion,
                    method: &Method, path: &str, query: Option<&str>) ->
        String {
        let authorization_raw = format!("[{}]", authorization);
        let bucket_raw = format!("[{}][{}][{}][{}]", version, method,
            path, query.unwrap_or(""));

        let auth_hash = Self::hash(&authorization_raw);
        let bucket_hash = Self::hash(&bucket_raw);

        debug!("Generated bucket: {} with hash: {}", bucket_raw, bucket_hash);

        Self::gen_ns_from_hash(shard, auth_hash.as_str(), bucket_hash.as_str())
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
            0, "", HttpVersion::Http11, &Method::Get, "/", Some("")),
            "bloom:0:90d52bc6:f773d6f1", "[shard=0][auth=no] HTTP/1.1 GET /");
        assert_eq!(CacheRoute::gen_ns(
            0, "", HttpVersion::Http11, &Method::Post, "/login", Some("")),
            "bloom:0:90d52bc6:afddff64",
            "[shard=0][auth=no] HTTP/1.1 POST /login");
        assert_eq!(CacheRoute::gen_ns(
            7, "8ab", HttpVersion::Http11, &Method::Options, "/feed", Some("")),
            "bloom:7:d42601a6:3352b2d5",
            "[shard=7][auth=yes] HTTP/1.1 OPTIONS /feed");
        assert_eq!(CacheRoute::gen_ns(
            80, "2d", HttpVersion::H2, &Method::Head, "/user", Some("u=1")),
            "bloom:80:471d2c40:e99cc313", "[shard=80][auth=yes] h2 HEAD /feed");
        assert_eq!(ROUTE_HASH_SIZE, CacheRoute::hash("7gCq81kzO5").len(),
            "Route size should be 8 (dynamic)");
    }
}

