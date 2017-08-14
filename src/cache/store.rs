// Bloom
//
// HTTP REST API caching middleware
// Copyright: 2017, Valerian Saliou <valerian@valeriansaliou.name>
// License: Mozilla Public License v2.0 (MPL v2.0)

use std::cmp;
use std::default::Default;

use r2d2::Pool;
use r2d2_redis::RedisConnectionManager;
use redis::Commands;
use futures::future;
use futures::future::FutureResult;

use APP_CONF;

pub struct CacheStoreBuilder;

pub struct CacheStore {
    pool: Pool<RedisConnectionManager>,
}

type CacheResult = FutureResult<Option<String>, &'static str>;

impl CacheStoreBuilder {
    pub fn new() -> CacheStore {
        info!("binding to store backend at {}", APP_CONF.redis.inet);

        let tcp_addr_raw = format!(
            "redis://{}:{}/{}",
            APP_CONF.redis.inet.ip(),
            APP_CONF.redis.inet.port(),
            APP_CONF.redis.database,
        );

        match RedisConnectionManager::new(tcp_addr_raw.as_ref()) {
            Ok(manager) => {
                match Pool::new(Default::default(), manager) {
                    Ok(pool) => {
                        info!("bound to store backend");

                        CacheStore { pool: pool }
                    }
                    Err(_) => panic!("could not spawn redis pool"),
                }
            },
            Err(_) => panic!("could not create redis connection manager"),
        }
    }
}

impl CacheStore {
    pub fn ensure(&self) -> CacheResult {
        let result = match self.pool.get() {
            Ok(_) => Ok(None),
            _ => Err("disconnected"),
        };

        future::result(result)
    }

    pub fn get(&self, key: &str) -> CacheResult {
        let result = match self.pool.get() {
            Ok(client) => {
                match (*client).get(key) {
                    Ok(string) => Ok(Some(string)),
                    _ => Err("failed"),
                }
            }
            _ => Err("disconnected"),
        };

        future::result(result)
    }

    pub fn set(&self, key: &str, value: &str, ttl: usize) -> CacheResult {
        let result = match self.pool.get() {
            Ok(client) => {
                // Cap TTL to 'max_key_expiration'
                let ttl_cap = cmp::min(ttl, APP_CONF.redis.max_key_expiration);

                // Ensure value is not larger than 'max_key_size'
                if value.len() > APP_CONF.redis.max_key_size {
                    Err("too large")
                } else {
                    match (*client).set_ex::<_, _, ()>(key, value, ttl_cap) {
                        Ok(_) => Ok(None),
                        _ => Err("failed"),
                    }
                }
            }
            _ => Err("disconnected"),
        };

        future::result(result)
    }

    pub fn purge(&self, key: &str) -> CacheResult {
        let result = match self.pool.get() {
            Ok(client) => {
                match (*client).del::<_, ()>(key) {
                    Ok(_) => Ok(None),
                    _ => Err("failed"),
                }
            }
            _ => Err("disconnected"),
        };

        future::result(result)
    }
}
