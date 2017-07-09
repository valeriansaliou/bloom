// Bloom
//
// HTTP REST API caching middleware
// Copyright: 2017, Valerian Saliou <valerian@valeriansaliou.name>
// License: Mozilla Public License v2.0 (MPL v2.0)

use std::cmp;
use std::sync::Arc;

use bmemcached::MemcachedClient;

use ::APP_CONF;

pub struct CacheStoreBuilder;

pub struct CacheStore {
    // TODO: not event required to Arc there...
    client: Option<Arc<MemcachedClient>>
}

type CacheResult = Result<Option<String>, &'static str>;

impl CacheStoreBuilder {
    pub fn new() -> CacheStore {
        CacheStore {
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

        info!("Binding to store backend at {}", APP_CONF.memcached.inet);

        let tcp_addr = format!("{}:{}", APP_CONF.memcached.inet.ip(),
                            APP_CONF.memcached.inet.port());

        match MemcachedClient::new(
            vec![tcp_addr], APP_CONF.memcached.pool_size) {
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
                                APP_CONF.memcached.max_key_expiration);

                // Ensure value is not larger than 'max_key_size'
                if value.len() > APP_CONF.memcached.max_key_size {
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
