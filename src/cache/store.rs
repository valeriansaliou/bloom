// Bloom
//
// HTTP REST API caching middleware
// Copyright: 2017, Valerian Saliou <valerian@valeriansaliou.name>
// License: Mozilla Public License v2.0 (MPL v2.0)

use std::cmp;
use std::time::Duration;

use r2d2::Pool;
use r2d2::config::Config;
use r2d2_redis::{RedisConnectionManager, Error};
use redis::{Connection, Commands};
use futures::future;
use futures::future::FutureResult;

use APP_CONF;

pub struct CacheStoreBuilder;

pub struct CacheStore {
    pool: Pool<RedisConnectionManager>,
}

#[derive(Debug)]
pub enum CacheStoreError {
    Disconnected,
    Failed,
    TooLarge,
}

type CacheResult = FutureResult<Option<String>, CacheStoreError>;

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
                let config = Config::<Connection, Error>::builder()
                    .test_on_check_out(false)
                    .pool_size(APP_CONF.redis.pool_size)
                    .idle_timeout(Some(Duration::from_secs(APP_CONF.redis.idle_timeout_seconds)))
                    .connection_timeout(Duration::from_secs(
                        APP_CONF.redis.connection_timeout_seconds))
                    .build();

                match Pool::new(config, manager) {
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
            _ => Err(CacheStoreError::Disconnected),
        };

        future::result(result)
    }

    pub fn get(&self, key: &str) -> CacheResult {
        let result = match self.pool.get() {
            Ok(client) => {
                match (*client).get(key) {
                    Ok(string) => Ok(Some(string)),
                    _ => Err(CacheStoreError::Failed),
                }
            }
            _ => Err(CacheStoreError::Disconnected),
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
                    Err(CacheStoreError::TooLarge)
                } else {
                    match (*client).set_ex::<_, _, ()>(key, value, ttl_cap) {
                        Ok(_) => Ok(None),
                        _ => Err(CacheStoreError::Failed),
                    }
                }
            }
            _ => Err(CacheStoreError::Disconnected),
        };

        future::result(result)
    }

    pub fn purge(&self, key: &str) -> CacheResult {
        let result = match self.pool.get() {
            Ok(client) => {
                match (*client).del::<_, ()>(key) {
                    Ok(_) => Ok(None),
                    _ => Err(CacheStoreError::Failed),
                }
            }
            _ => Err(CacheStoreError::Disconnected),
        };

        future::result(result)
    }
}
