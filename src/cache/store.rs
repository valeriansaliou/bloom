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
        // TODO

        info!("Binding to store backend at {}", self.config_memcached.inet);
    }
}
