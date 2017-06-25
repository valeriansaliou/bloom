// Bloom
//
// HTTP REST API caching middleware
// Copyright: 2017, Valerian Saliou <valerian@valeriansaliou.name>
// License: Mozilla Public License v2.0 (MPL v2.0)

extern crate hyper;
extern crate farmhash;

use self::hyper::Method;

pub struct CacheRead;

impl CacheRead {
    pub fn gen_ns(shard: u8, method: &Method, path: &str, authorization: String)
        -> String {
        let namespace_raw = format!("[{}][{}][{}]", method, path,
            authorization);
        let namespace_hash = farmhash::hash64(namespace_raw.as_bytes());

        debug!("Generated namespace: {} with hash: {}", namespace_raw,
            namespace_hash);

        format!("{}.{:x}", shard, namespace_hash)
    }

    pub fn acquire(ns: &str) {
        // TODO: Not implemented
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn it_generates_valid_ns() {
        assert_eq!(CacheRead::gen_ns(0, &Method::Get, "/", ""),
            "0.d1502b360785d097", "[shard=0][auth=anonymous] GET /");
        assert_eq!(CacheRead::gen_ns(0, &Method::Post, "/login", ""),
            "0.899c4e4e1578071f", "[shard=0][auth=anonymous] POST /login");
        assert_eq!(CacheRead::gen_ns(7, &Method::Options, "/feed", "8ab"),
            "7.fe47c58fc02f4efc", "[shard=7][auth=8ab] OPTIONS /feed");
    }
}
