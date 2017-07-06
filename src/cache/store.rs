// Bloom
//
// HTTP REST API caching middleware
// Copyright: 2017, Valerian Saliou <valerian@valeriansaliou.name>
// License: Mozilla Public License v2.0 (MPL v2.0)

use std::sync::atomic::{AtomicBool, Ordering};

use memcached::Client;
use memcached::proto::ProtoType;

use config::config::ConfigMemcached;

pub struct CacheStoreBuilder;

pub struct CacheStore {
    config_memcached: ConfigMemcached,
    is_connected: AtomicBool
    // client: Client  <-- TODO: impl. clone for Client?
}

// type CacheResult = impl Future<Item = bool>;

impl CacheStoreBuilder {
    pub fn new(config_memcached: ConfigMemcached) -> CacheStore {
        CacheStore {
            config_memcached: config_memcached,
            is_connected: AtomicBool::new(false)
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

        let tcp_addr = format!("tcp://{}:{}", self.config_memcached.inet.ip(),
            self.config_memcached.inet.port());
        let servers = [(tcp_addr.as_str(), 1)];

        match Client::connect(&servers, ProtoType::Binary) {
            Ok(client) => {
                // TODO: assign to struct
                // self.client = client

                self.is_connected.store(true, Ordering::Relaxed);
            }
            Err(err) => panic!("could not connect to memcached: {}", err)
        }

        info!("Bound to store backend");
    }

    pub fn get(&self, key: &str) {
        if self.is_connected.load(Ordering::Relaxed) == true {
            // TODO

            // futures::future::ok(true)
        }

        // futures::future::err(false)

        // TODO: return future immediately if disconnected (w/ 'false' value)
        // TODO: get and return a future (w/ 'true' value or 'false if fail)
    }

    pub fn set(&self, key: &str, value: &str, ttl: u32) {
        if self.is_connected.load(Ordering::Relaxed) == true {
            // TODO

            // futures::future::ok(true)
        }

        // futures::future::err(false)

        // TODO: return future immediately if disconnected (w/ 'false' value)

        // TODO: set and return a future (needed? maybe we dont even need to \
        //   ack as this is best effort, maybe just log write errors) \
        //   (w/ 'true' value or 'false if fail)
        // TODO: value maybe would be better be a stream to avoid large buffers

        // TODO: enforce config values:
        //   - ConfigMemcached.max_key_size
        //   - ConfigMemcached.max_key_expiration
    }

    pub fn purge(&self, key: &str) {
        if self.is_connected.load(Ordering::Relaxed) == true {
            // TODO

            // futures::future::ok(true)
        }
    }
}
