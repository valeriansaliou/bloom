// Bloom
//
// HTTP REST API caching middleware
// Copyright: 2017, Valerian Saliou <valerian@valeriansaliou.name>
// License: Mozilla Public License v2.0 (MPL v2.0)

use farmhash;
use http::{Method, Version};

pub struct CacheRoute;

pub const ROUTE_HASH_SIZE: usize = 8;

pub static ROUTE_PREFIX: &'static str = "bloom";

impl CacheRoute {
    pub fn gen_key_cache_from_hash(
        shard: u8,
        auth_hash: &str,
        route_hash: &str,
    ) -> (String, String) {
        let mask = format!("{}:{}", auth_hash, route_hash);

        (format!("{}:{}:c:{}", ROUTE_PREFIX, shard, &mask), mask)
    }

    pub fn gen_key_auth_from_hash(shard: u8, auth_hash: &str) -> (String, String) {
        let mask = format!("a:{}", auth_hash);

        (format!("{}:{}:{}", ROUTE_PREFIX, shard, mask), mask)
    }

    pub fn gen_key_bucket_from_hash(shard: u8, bucket_hash: &str) -> (String, String) {
        let mask = format!("b:{}", bucket_hash);

        (format!("{}:{}:{}", ROUTE_PREFIX, shard, mask), mask)
    }

    pub fn gen_key_cache(
        shard: u8,
        auth_hash: &str,
        version: Version,
        method: &Method,
        path: &str,
        query: Option<&str>,
        origin: Option<&str>,
    ) -> (String, String) {
        let version_str = match version {
            Version::HTTP_09 => "HTTP/0.9",
            Version::HTTP_10 => "HTTP/1.0",
            Version::HTTP_11 => "HTTP/1.1",
            Version::HTTP_2 => "h2",
            Version::HTTP_3 => "h3",
            _ => "HTTP/1.1",
        };

        let bucket_raw = format!(
            "[{}|{}|{}|{}|{}]",
            version_str,
            method,
            path,
            query.unwrap_or(""),
            origin.unwrap_or("null"),
        );

        let route_hash = Self::hash(&bucket_raw);

        debug!("generated bucket: {} with hash: {}", bucket_raw, route_hash);

        Self::gen_key_cache_from_hash(shard, auth_hash, &route_hash)
    }

    pub fn hash(value: &str) -> String {
        debug!("hashing value: {}", value);

        format!("{:x}", farmhash::fingerprint32(value.as_bytes()))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn it_generates_valid_ns() {
        assert_eq!(
            CacheRoute::gen_key_cache(
                0,
                "dc56d17a",
                Version::HTTP_11,
                &Method::GET,
                "/",
                Some(""),
                None,
            ),
            (
                "bloom:0:c:dc56d17a:e6a8b05d".to_string(),
                "dc56d17a:e6a8b05d".to_string(),
            ),
            "[shard=0][auth=no] HTTP/1.1 GET /"
        );
        assert_eq!(
            CacheRoute::gen_key_cache(
                0,
                "dc56d17a",
                Version::HTTP_11,
                &Method::POST,
                "/login",
                Some(""),
                None,
            ),
            (
                "bloom:0:c:dc56d17a:fbdc5f7c".to_string(),
                "dc56d17a:fbdc5f7c".to_string(),
            ),
            "[shard=0][auth=no] HTTP/1.1 POST /login"
        );
        assert_eq!(
            CacheRoute::gen_key_cache(
                7,
                "6d0f1448",
                Version::HTTP_11,
                &Method::OPTIONS,
                "/feed",
                Some(""),
                None,
            ),
            (
                "bloom:7:c:6d0f1448:2f484c4a".to_string(),
                "6d0f1448:2f484c4a".to_string(),
            ),
            "[shard=7][auth=yes] HTTP/1.1 OPTIONS /feed"
        );
        assert_eq!(
            CacheRoute::gen_key_cache(
                80,
                "d73f0f31",
                Version::HTTP_2,
                &Method::HEAD,
                "/user",
                Some("u=1"),
                Some("https://valeriansaliou.name"),
            ),
            (
                "bloom:80:c:d73f0f31:e186dab7".to_string(),
                "d73f0f31:e186dab7".to_string(),
            ),
            "[shard=80][auth=yes] h2 HEAD /feed"
        );
        assert_eq!(
            ROUTE_HASH_SIZE,
            CacheRoute::hash("7gCq81kzO5").len(),
            "Route size should be 8 (dynamic)"
        );
    }
}
