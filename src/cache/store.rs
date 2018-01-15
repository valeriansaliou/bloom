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
use redis::{self, Value, Connection, Commands, PipelineCommands};
use futures::future::Future;
use futures_cpupool::CpuPool;

use super::route::ROUTE_PREFIX;
use APP_CONF;

static KEY_BODY: &'static str = "b";
static KEY_FINGERPRINT: &'static str = "f";
static KEY_TAGS: &'static str = "t";
static KEY_TAGS_SEPARATOR: &'static str = ",";

lazy_static! {
    pub static ref EXECUTOR_POOL: CpuPool = CpuPool::new(APP_CONF.cache.executor_pool as usize);
}

pub struct CacheStoreBuilder;

pub struct CacheStore {
    pool: Pool<RedisConnectionManager>,
}

#[derive(Debug)]
pub enum CacheStoreError {
    Disconnected,
    Failed,
    Invalid,
    Corrupted,
    Partial,
    TooLarge,
}

#[derive(Debug)]
pub enum CachePurgeVariant {
    Bucket,
    Auth,
}

type CacheReadResultFuture = Box<Future<Item = Option<String>, Error = CacheStoreError>>;
type CacheWriteResult = Result<(String), (CacheStoreError, String)>;
type CacheWriteResultFuture = Box<Future<Item = CacheWriteResult, Error = ()>>;
type CachePurgeResult = Result<(), CacheStoreError>;

impl CacheStoreBuilder {
    pub fn new() -> CacheStore {
        info!(
            "binding to store backend at {}:{}",
            APP_CONF.redis.host,
            APP_CONF.redis.port
        );

        let addr_auth = match APP_CONF.redis.password {
            Some(ref password) => format!(":{}@", password),
            None => "".to_string(),
        };

        let tcp_addr_raw =
            format!(
            "redis://{}{}:{}/{}",
            &addr_auth,
            APP_CONF.redis.host,
            APP_CONF.redis.port,
            APP_CONF.redis.database,
        );

        debug!("will connect to redis at: {}", tcp_addr_raw);

        match RedisConnectionManager::new(tcp_addr_raw.as_ref()) {
            Ok(manager) => {
                let config = Config::<Connection, Error>::builder()
                    .initialization_fail_fast(false)
                    .test_on_check_out(false)
                    .pool_size(APP_CONF.redis.pool_size)
                    .max_lifetime(Some(
                        Duration::from_secs(APP_CONF.redis.max_lifetime_seconds),
                    ))
                    .idle_timeout(Some(
                        Duration::from_secs(APP_CONF.redis.idle_timeout_seconds),
                    ))
                    .connection_timeout(Duration::from_secs(
                        APP_CONF.redis.connection_timeout_seconds,
                    ))
                    .build();

                match Pool::new(config, manager) {
                    Ok(pool) => {
                        info!("bound to store backend");

                        CacheStore { pool: pool }
                    }
                    Err(_) => panic!("could not spawn redis pool"),
                }
            }
            Err(_) => panic!("could not create redis connection manager"),
        }
    }
}

impl CacheStore {
    pub fn get_meta(&self, shard: u8, key: String) -> CacheReadResultFuture {
        let pool = self.pool.to_owned();

        Box::new(EXECUTOR_POOL.spawn_fn(move || {
            get_cache_store_client!(pool, CacheStoreError::Disconnected, client {
                    match (*client).hget::<_, _, (Value, Value)>(key, (KEY_FINGERPRINT, KEY_TAGS)) {
                        Ok(value) => {
                            match value {
                                (Value::Data(fingerprint_bytes), tags_bytes) => {
                                    // Parse tags and bump their last access time
                                    if let Value::Data(tags_bytes_data) = tags_bytes {
                                        if let Ok(tags_data) = String::from_utf8(
                                            tags_bytes_data) {
                                            if tags_data.is_empty() == false {
                                                let tags = tags_data.split(KEY_TAGS_SEPARATOR)
                                                    .map(|tag| {
                                                        format!(
                                                            "{}:{}:{}", ROUTE_PREFIX, shard, tag
                                                        )
                                                    })
                                                    .collect::<Vec<String>>();

                                                // Proceed a soft bump of last access time of \
                                                //   associated tag keys. This prevents a \
                                                //   frequently accessed cache namespace to \
                                                //   become 'orphan' (ie. one or more tag keys \
                                                //   are LRU-expired), and thus cache namespace \
                                                //   not to be properly removed on purge of an \
                                                //   associated tag.
                                                // Also, count bumped keys. It may happen that \
                                                //   some tag keys are incorrectly removed by \
                                                //   Redis LRU system, as it is probabilistic \
                                                //   and thus might sample some keys incorrectly.
                                                // The conditions explained above only happens on \
                                                //   Redis instances with used memory going over \
                                                //   the threshold of the max memory policy.
                                                let tags_count = tags.len();

                                                match redis::cmd("TOUCH").arg(tags)
                                                    .query::<usize>(&*client) {
                                                    Ok(bump_count) => {
                                                        // Partial bump count? Do not serve cache.
                                                        if bump_count < tags_count {
                                                            return Err(CacheStoreError::Partial)
                                                        }
                                                    },
                                                    Err(err) => {
                                                        error!(
                                                            "error bumping access time of tags: {}",
                                                            err
                                                        );
                                                    }
                                                }
                                            }
                                        }
                                    }

                                    // Decode raw bytes to string
                                    if let Ok(fingerprint) = String::from_utf8(fingerprint_bytes) {
                                        Ok(Some(fingerprint))
                                    } else {
                                        Err(CacheStoreError::Corrupted)
                                    }
                                },
                                (Value::Nil, _) | (_, Value::Nil) => Ok(None),
                                _ => Err(CacheStoreError::Invalid),
                            }
                        },
                        _ => Err(CacheStoreError::Failed),
                    }
                })
        }))
    }

    pub fn get_body(&self, key: String) -> CacheReadResultFuture {
        let pool = self.pool.to_owned();

        Box::new(EXECUTOR_POOL.spawn_fn(move || {
            get_cache_store_client!(pool, CacheStoreError::Disconnected, client {
                    match (*client).hget::<_, _, Value>(key, KEY_BODY) {
                        Ok(value) => {
                            match value {
                                Value::Data(body_bytes) => {
                                    // Decode raw bytes to string
                                    if let Ok(body) = String::from_utf8(body_bytes) {
                                        Ok(Some(body))
                                    } else {
                                        Err(CacheStoreError::Corrupted)
                                    }
                                },
                                Value::Nil => Ok(None),
                                _ => Err(CacheStoreError::Invalid),
                            }
                        },
                        _ => Err(CacheStoreError::Failed),
                    }
                })
        }))
    }

    pub fn set(
        &self,
        key: String,
        key_mask: String,
        value: String,
        fingerprint: String,
        ttl: usize,
        key_tags: Vec<(String, String)>,
    ) -> CacheWriteResultFuture {
        let pool = self.pool.to_owned();

        Box::new(EXECUTOR_POOL.spawn_fn(move || {
            Ok(get_cache_store_client!(
                    pool,
                    (CacheStoreError::Disconnected, fingerprint),

                    client {
                        // Cap TTL to 'max_key_expiration'
                        let ttl_cap = cmp::min(ttl, APP_CONF.redis.max_key_expiration);

                        // Ensure value is not larger than 'max_key_size'
                        if value.len() > APP_CONF.redis.max_key_size {
                            Err((CacheStoreError::TooLarge, fingerprint))
                        } else {
                            let mut pipeline = redis::pipe();

                            // Append storage command
                            {
                                let key_tag_masks = key_tags.iter()
                                    .map(|key_tag| key_tag.1.as_ref())
                                    .collect::<Vec<&str>>();

                                pipeline.hset_multiple(
                                    &key, &[
                                        (KEY_FINGERPRINT, &fingerprint),
                                        (KEY_TAGS, &key_tag_masks.join(KEY_TAGS_SEPARATOR)),
                                        (KEY_BODY, &value)
                                    ]
                                ).ignore();
                            }

                            pipeline.expire(&key, ttl_cap).ignore();

                            for key_tag in key_tags {
                                pipeline.sadd(&key_tag.0, &key_mask).ignore();
                                pipeline.expire(&key_tag.0, APP_CONF.redis.max_key_expiration);
                            }

                            // Bucket (MULTI operation for main data + bucket marker)
                            match pipeline.query::<()>(&*client) {
                                Ok(_) => Ok(fingerprint),
                                Err(err) => {
                                    error!("got store error: {}", err);

                                    Err((CacheStoreError::Failed, fingerprint))
                                }
                            }
                        }
                    }
                ))
        }))
    }

    pub fn purge_tag(
        &self,
        variant: &CachePurgeVariant,
        shard: u8,
        key_tag: &str,
    ) -> CachePurgeResult {
        get_cache_store_client!(self.pool, CacheStoreError::Disconnected, client {
            // Invoke keyspace cleanup script for key tag
            let result = redis::Script::new(variant.get_script())
                .arg(ROUTE_PREFIX)
                .arg(shard)
                .arg(key_tag)
                .invoke::<()>(&*client);

            result
                .and(Ok(()))
                .or(Err(CacheStoreError::Failed))
        })
    }
}

impl CachePurgeVariant {
    fn get_script(&self) -> &'static str {
        match *self {
            CachePurgeVariant::Bucket |
            CachePurgeVariant::Auth => {
                r#"
                    local targets = {}

                    for _, tag in pairs(redis.call('SMEMBERS', ARGV[3])) do
                        table.insert(targets, ARGV[1] .. ":" .. ARGV[2] .. ":c:" .. tag)
                    end

                    table.insert(targets, ARGV[3])

                    redis.call('DEL', unpack(targets))
                "#
            }
        }
    }
}
