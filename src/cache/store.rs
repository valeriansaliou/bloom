// Bloom
//
// HTTP REST API caching middleware
// Copyright: 2017, Valerian Saliou <valerian@valeriansaliou.name>
// License: Mozilla Public License v2.0 (MPL v2.0)

use std::cmp;
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};

use bmemcached::MemcachedClient;
use bmemcached::errors::BMemcachedError;

use config::config::ConfigMemcached;

pub struct CacheStoreBuilder;

pub struct CacheStore {
    config_memcached: ConfigMemcached,
    client: Option<Arc<MemcachedClient>>
}

type CacheResult = Result<Option<String>, &'static str>;

impl CacheStoreBuilder {
    pub fn new(config_memcached: ConfigMemcached) -> CacheStore {
        CacheStore {
            config_memcached: config_memcached,
            client: None
        }
    }
}

impl CacheStore {
    pub fn bind(&mut self) {
        // TODO: enforce config values:
        //   - ConfigMemcached.reconnect
        //   - ConfigMemcached.timeout

        // TODO: ensure following contracts:
            // if first connect fails, panic!()
            // if connection to memcached is lost at any point, mark as \
            //   disconnected and immediately return get/set futures w/o \
            //   trying to access the network (this doesnt add extra-latency \
            //   to api requests). but: keep trying to reconnect in bg.
            //   (best-effort retries, hit the api directly in that case and \
            //   return response w/ the DIRECT bloom status header)

        info!("Binding to store backend at {}", self.config_memcached.inet);

        let tcp_addr = format!("{}:{}", self.config_memcached.inet.ip(),
                            self.config_memcached.inet.port());

        match MemcachedClient::new(
            vec![tcp_addr], self.config_memcached.pool_size) {
            Ok(client_raw) => {
                self.client = Some(Arc::new(client_raw));
            }
            Err(err) => panic!("could not connect to memcached")
        }

        info!("Bound to store backend");
    }

    pub fn get(&self, key: &str) -> CacheResult {
        match self.client {
            Some(ref client) => {
                match client.get(key) {
                    Ok(String) => Ok(Some(String)),
                    _ => Err("failed")
                }
            }
            _ => {
                Err("disconnected")
            }
        }
    }

    pub fn set(&self, key: &str, value: &str, ttl: u32) -> CacheResult {
        match self.client {
            Some(ref client) => {
                // Cap TTL to 'max_key_expiration'
                let ttl_cap = cmp::min(ttl,
                                self.config_memcached.max_key_expiration);

                // Ensure value is not larger than 'max_key_size'
                if value.len() > self.config_memcached.max_key_size {
                    return Err("too large")
                }

                match client.set(key, value, ttl_cap) {
                    Ok(_) => Ok(None),
                    _ => Err("failed")
                }
            }
            _ => {
                Err("disconnected")
            }
        }
    }

    pub fn purge(&self, key: &str) -> CacheResult {
        match self.client {
            Some(ref client) => {
                match client.delete(key) {
                    Ok(_) => Ok(None),
                    _ => Err("failed")
                }
            }
            _ => {
                Err("disconnected")
            }
        }
    }
}
