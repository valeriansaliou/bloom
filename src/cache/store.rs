// Bloom
//
// HTTP REST API caching middleware
// Copyright: 2017, Valerian Saliou <valerian@valeriansaliou.name>
// License: Mozilla Public License v2.0 (MPL v2.0)

use std::cmp;

use bmemcached::MemcachedClient;
use futures::future;
use futures::future::FutureResult;

use ::APP_CONF;

pub struct CacheStoreBuilder;

pub struct CacheStore {
    client: Option<MemcachedClient>
}

type CacheResult = FutureResult<Option<String>, &'static str>;

impl CacheStoreBuilder {
    pub fn new() -> CacheStore {
        info!("binding to store backend at {}", APP_CONF.memcached.inet);

        let tcp_addr = format!("{}:{}", APP_CONF.memcached.inet.ip(),
                            APP_CONF.memcached.inet.port());

        match MemcachedClient::new(
            vec![tcp_addr], APP_CONF.memcached.pool_size) {
            Ok(client_raw) => {
                info!("bound to store backend");

                CacheStore {
                    client: Some(client_raw)
                }
            }
            Err(_) => panic!("could not connect to memcached")
        }
    }
}

impl CacheStore {
    pub fn get(&self, key: &str) -> CacheResult {
        let result = match self.client {
            Some(ref client) => {
                match client.get(key) {
                    Ok(string) => Ok(Some(string)),
                    _ => Err("failed")
                }
            }
            _ => {
                Err("disconnected")
            }
        };

        future::result(result)
    }

    pub fn set(&self, key: &str, value: &str, ttl: u32) -> CacheResult {
        let result = match self.client {
            Some(ref client) => {
                // Cap TTL to 'max_key_expiration'
                let ttl_cap = cmp::min(ttl,
                                APP_CONF.memcached.max_key_expiration);

                // Ensure value is not larger than 'max_key_size'
                if value.len() > APP_CONF.memcached.max_key_size {
                    Err("too large")
                } else {
                    match client.set(key, value, ttl_cap) {
                        Ok(_) => Ok(None),
                        _ => Err("failed")
                    }
                }
            }
            _ => {
                Err("disconnected")
            }
        };

        future::result(result)
    }

    pub fn purge(&self, key: &str) -> CacheResult {
        let result = match self.client {
            Some(ref client) => {
                match client.delete(key) {
                    Ok(_) => Ok(None),
                    _ => Err("failed")
                }
            }
            _ => {
                Err("disconnected")
            }
        };

        future::result(result)
    }
}

#[cfg(test)]
mod tests {
    use futures::Future;
    use super::*;

    fn get_client() -> CacheStore {
        CacheStore{
            client: None
        }
    }

    #[test]
    #[should_panic]
    fn it_fails_connecting_to_cache() {
        assert!(CacheStoreBuilder::new().client.is_none());
    }

    #[test]
    fn it_fails_getting_cache() {
        assert!(get_client().get("bloom:0:90d52bc6:f773d6f1").wait().is_err());
    }

    #[test]
    fn it_fails_setting_cache() {
        assert!(get_client().set("bloom:0:90d52bc6:f773d6f1", "{}", 30).wait()
            .is_err());
    }

    #[test]
    fn it_fails_purging_cache() {
        assert!(get_client().purge("bloom:0:90d52bc6:f773d6f1").wait().is_err());
    }
}
