// Bloom
//
// HTTP REST API caching middleware
// Copyright: 2017, Valerian Saliou <valerian@valeriansaliou.name>
// License: Mozilla Public License v2.0 (MPL v2.0)

use config::config::ConfigMemcached;

pub struct CacheStoreBuilder;

#[derive(Clone)]
pub struct CacheStore {
    config_memcached: ConfigMemcached
}

impl CacheStoreBuilder {
    pub fn new(config_memcached: ConfigMemcached) -> CacheStore {
        CacheStore {
            config_memcached: config_memcached
        }
    }
}

impl CacheStore {
    pub fn bind(&self) {
        // TODO: bind to ConfigMemcached.inet

        // TODO: enforce config values:
        //   - ConfigMemcached.pool_size
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
    }

    pub fn get(&self, key: &str) {
        // TODO: return future immediately if disconnected (w/ 'false' value)
        // TODO: get and return a future (w/ 'true' value or 'false if fail)
    }

    pub fn set(&self, key: &str, value: &str, ttl: u32) {
        // TODO: return future immediately if disconnected (w/ 'false' value)

        // TODO: set and return a future (needed? maybe we dont even need to \
        //   ack as this is best effort, maybe just log write errors) \
        //   (w/ 'true' value or 'false if fail)
        // TODO: value maybe would be better be a stream to avoid large buffers

        // TODO: enforce config values:
        //   - ConfigMemcached.max_key_size
        //   - ConfigMemcached.max_key_expiration
    }
}
