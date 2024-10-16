// Bloom
//
// HTTP REST API caching middleware
// Copyright: 2017, Valerian Saliou <valerian@valeriansaliou.name>
// License: Mozilla Public License v2.0 (MPL v2.0)

use brotli::{CompressorReader as BrotliCompressor, Decompressor as BrotliDecompressor};
use futures::future::Future;
use futures_cpupool::CpuPool;
use r2d2::Pool;
use redis::{self, Commands, Value};
use std::cmp;
use std::io::Read;
use std::time::Duration;

use super::route::ROUTE_PREFIX;
use crate::APP_CONF;

pub const BODY_COMPRESS_RATIO: u32 = 5;

static KEY_BODY: &str = "b";
static KEY_FINGERPRINT: &str = "f";
static KEY_TAGS: &str = "t";
static KEY_TAGS_SEPARATOR: &str = ",";

lazy_static! {
    pub static ref EXECUTOR_POOL: CpuPool = CpuPool::new(APP_CONF.cache.executor_pool as usize);
}

pub struct CacheStoreBuilder;

pub struct CacheStore {
    pool: Pool<redis::Client>,
}

#[derive(Debug)]
pub enum CacheStoreError {
    Disconnected,
    Failed,
    Invalid,
    Corrupted,
    TooLarge,
}

#[derive(Debug)]
pub enum CachePurgeVariant {
    Bucket,
    Auth,
}

type CacheReadResultFuture = Box<dyn Future<Item = Option<String>, Error = CacheStoreError>>;
type CacheWriteResult = Result<String, (CacheStoreError, String)>;
type CacheWriteResultFuture = Box<dyn Future<Item = CacheWriteResult, Error = ()>>;
type CachePurgeResult = Result<(), CacheStoreError>;

impl CacheStoreBuilder {
    pub fn create() -> CacheStore {
        info!(
            "binding to store backend at {}:{}",
            APP_CONF.redis.host, APP_CONF.redis.port
        );

        let addr_auth = APP_CONF
            .redis
            .password
            .as_ref()
            .map_or_else(String::new, |password| format!(":{password}@"));

        let tcp_addr_raw = format!(
            "redis://{}{}:{}/{}",
            &addr_auth, APP_CONF.redis.host, APP_CONF.redis.port, APP_CONF.redis.database,
        );

        debug!("will connect to redis at: {}", tcp_addr_raw);

        match redis::Client::open(tcp_addr_raw.as_ref()) {
            Ok(manager) => {
                let builder = Pool::builder()
                    .test_on_check_out(false)
                    .max_size(APP_CONF.redis.pool_size)
                    .max_lifetime(Some(Duration::from_secs(
                        APP_CONF.redis.max_lifetime_seconds,
                    )))
                    .idle_timeout(Some(Duration::from_secs(
                        APP_CONF.redis.idle_timeout_seconds,
                    )))
                    .connection_timeout(Duration::from_secs(
                        APP_CONF.redis.connection_timeout_seconds,
                    ));

                match builder.build(manager) {
                    Ok(pool) => {
                        info!("bound to store backend");

                        CacheStore { pool }
                    }
                    Err(e) => panic!("could not spawn redis pool: {e}"),
                }
            }
            Err(e) => panic!("could not create redis connection manager: {e}"),
        }
    }
}

impl CacheStore {
    pub fn get_meta(&self, shard: u8, key: String) -> CacheReadResultFuture {
        let pool = self.pool.clone();

        Box::new(EXECUTOR_POOL.spawn_fn(move || {
            get_cache_store_client_try!(pool, CacheStoreError::Disconnected, client {
                match (*client).hget::<_, _, (Value, Value)>(key, (KEY_FINGERPRINT, KEY_TAGS)) {
                    Ok(value) => {
                        match value {
                            (Value::Data(fingerprint_bytes), tags_bytes) => {
                                // Parse tags and bump their last access time
                                if let Value::Data(tags_bytes_data) = tags_bytes {
                                    if let Ok(tags_data) = String::from_utf8(
                                        tags_bytes_data) {
                                        if !tags_data.is_empty() {
                                            let tags = tags_data.split(KEY_TAGS_SEPARATOR)
                                                .map(|tag| {
                                                    format!(
                                                        "{ROUTE_PREFIX}:{shard}:{tag}"
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
                                                .query::<usize>(&mut *client) {
                                                Ok(bump_count) => {
                                                    // Partial bump count? Consider cache as \
                                                    //   non-existing
                                                    if bump_count < tags_count {
                                                        info!(
                                                            "got only partial tag count: {}/{}",
                                                            bump_count, tags_count
                                                        );

                                                        return Ok(None);
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
                            (Value::Nil, _) => Ok(None),
                            _ => Err(CacheStoreError::Invalid),
                        }
                    },
                    _ => Err(CacheStoreError::Failed),
                }
            })
        }))
    }

    pub fn get_body(&self, key: String) -> CacheReadResultFuture {
        let pool = self.pool.clone();

        Box::new(EXECUTOR_POOL.spawn_fn(move || {
            get_cache_store_client_try!(pool, CacheStoreError::Disconnected, client {
                match (*client).hget::<_, _, Value>(key, KEY_BODY) {
                    Ok(value) => {
                        match value {
                            Value::Data(body_bytes_raw) => {
                                let body_bytes_result =
                                if APP_CONF.cache.compress_body {
                                    // Decompress raw bytes
                                    let mut decompressor = BrotliDecompressor::new(
                                        &body_bytes_raw[..], 4096
                                    );

                                    let mut decompress_bytes = Vec::new();

                                    match decompressor.read_to_end(&mut decompress_bytes) {
                                        Ok(_) => {
                                            if !body_bytes_raw.is_empty() &&
                                                    decompress_bytes.is_empty() {
                                                error!(
                                                    "decompressed store value has empty body"
                                                );

                                                Err(())
                                            } else {
                                                Ok(decompress_bytes)
                                            }
                                        },
                                        Err(err) => {
                                            error!("error decompressing store value: {}", err);

                                            Err(())
                                        }
                                    }
                                } else {
                                    Ok(body_bytes_raw)
                                };

                                // Decode raw bytes to string
                                if let Ok(body_bytes) = body_bytes_result {
                                    if let Ok(body) = String::from_utf8(body_bytes) {
                                        Ok(Some(body))
                                    } else {
                                        Err(CacheStoreError::Corrupted)
                                    }
                                } else {
                                    Err(CacheStoreError::Failed)
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
        let pool = self.pool.clone();

        Box::new(EXECUTOR_POOL.spawn_fn(move || {
            Ok(get_cache_store_client_try!(
                pool,
                (CacheStoreError::Disconnected, fingerprint),

                client {
                    // Cap TTL to 'max_key_expiration'
                    let ttl_cap = cmp::min(ttl, APP_CONF.redis.max_key_expiration);

                    // Ensure value is not larger than 'max_key_size'
                    if value.len() > APP_CONF.redis.max_key_size {
                        Err((CacheStoreError::TooLarge, fingerprint))
                    } else {
                        // Compress value?
                        let store_value_bytes_result = if APP_CONF.cache.compress_body {
                            let mut compressor = BrotliCompressor::new(
                                value.as_bytes(), 4096, BODY_COMPRESS_RATIO, 22
                            );

                            let mut compress_bytes = Vec::new();

                            match compressor.read_to_end(&mut compress_bytes) {
                                Ok(_) => Ok(compress_bytes),
                                Err(err) => {
                                    error!("error compressing store value: {}", err);

                                    Err(())
                                }
                            }
                        } else {
                            Ok(value.into_bytes())
                        };

                        if let Ok(store_value_bytes) = store_value_bytes_result {
                            let mut pipeline = redis::pipe();

                            // Append storage command
                            {
                                let key_tag_masks = key_tags.iter()
                                    .map(|key_tag| key_tag.1.as_ref())
                                    .collect::<Vec<&str>>();

                                pipeline.hset_multiple(
                                    &key, &[
                                        (
                                            KEY_FINGERPRINT,
                                            fingerprint.as_bytes()
                                        ),

                                        (
                                            KEY_TAGS,
                                            key_tag_masks.join(KEY_TAGS_SEPARATOR).as_bytes()
                                        ),

                                        (
                                            KEY_BODY,
                                            &store_value_bytes
                                        )
                                    ]
                                ).ignore();
                            }

                            pipeline.expire(&key, ttl_cap).ignore();

                            for key_tag in key_tags {
                                pipeline.sadd(&key_tag.0, &key_mask).ignore();
                                pipeline.expire(&key_tag.0, APP_CONF.redis.max_key_expiration);
                            }

                            // Bucket (MULTI operation for main data + bucket marker)
                            match pipeline.query::<()>(&mut *client) {
                                Ok(()) => Ok(fingerprint),
                                Err(err) => {
                                    error!("got store error: {}", err);

                                    Err((CacheStoreError::Failed, fingerprint))
                                }
                            }
                        } else {
                            error!("error generating store value");

                            Err((CacheStoreError::Failed, fingerprint))
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
        get_cache_store_client_wait!(self.pool, CacheStoreError::Disconnected, client {
            // Invoke keyspace cleanup script for key tag
            let result = redis::Script::new(variant.get_script())
                .arg(ROUTE_PREFIX)
                .arg(shard)
                .arg(key_tag)
                .invoke::<()>(&mut *client);

            result
                .and(Ok(()))
                .or(Err(CacheStoreError::Failed))
        })
    }
}

impl CachePurgeVariant {
    const fn get_script(&self) -> &'static str {
        // Notice: there is a limit of 1000 purgeable tags per bucket. Purging a lot of tags at \
        //   once is dangerous for Bloom, as the underlying Redis server is at risk of blocking.
        match *self {
            Self::Bucket | Self::Auth => {
                r#"
                  local batch_size = 1000
                  local cursor = "0"
                  local targets, result

                  repeat
                      targets = {}
                      result = redis.call('SSCAN', ARGV[3], cursor, 'COUNT', batch_size)
                      cursor = result[1]
                      for _, tag in ipairs(result[2]) do
                          table.insert(targets, ARGV[1] .. ":" .. ARGV[2] .. ":c:" .. tag)
                      end

                      if #targets > 0 then
                          redis.call('UNLINK', unpack(targets))
                      end
                  until cursor == "0"

                  redis.call('UNLINK', ARGV[3])
                "#
            }
        }
    }
}
