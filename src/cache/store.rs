// Bloom
//
// HTTP REST API caching middleware
// Copyright: 2017, Valerian Saliou <valerian@valeriansaliou.name>
// License: Mozilla Public License v2.0 (MPL v2.0)

use std::cmp;
use std::time::Duration;

use redis::aio::ConnectionManager;
use redis::{self, AsyncCommands, Client, Value};
use tokio::sync::OnceCell;

use super::route::ROUTE_PREFIX;
use crate::APP_CONF;

static KEY_BODY: &'static str = "b";
static KEY_FINGERPRINT: &'static str = "f";
static KEY_COMPRESSED: &'static str = "c";
static KEY_TAGS: &'static str = "t";
static KEY_TAGS_SEPARATOR: &'static str = ",";

static VALUE_COMPRESSED_YES: &'static [u8] = "1".as_bytes();
static VALUE_COMPRESSED_NO: &'static [u8] = "0".as_bytes();

pub struct CacheStoreBuilder;

pub struct CacheStore {
    client: Client,
    timeout: Duration,
    connections: CacheStoreConnections,
}

pub struct CacheStoreConnections {
    main: OnceCell<ConnectionManager>,
    scripts: OnceCell<ConnectionManager>,
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

type CacheWriteResult = Result<String, (CacheStoreError, String)>;
type CachePurgeResult = Result<(), CacheStoreError>;

impl CacheStoreBuilder {
    pub fn new() -> CacheStore {
        info!(
            "binding to store backend at {}:{}",
            APP_CONF.redis.host, APP_CONF.redis.port
        );

        let addr_auth = match APP_CONF.redis.password {
            Some(ref password) => format!(":{}@", password),
            None => "".to_string(),
        };

        let tcp_addr_raw = format!(
            "redis://{}{}:{}/{}",
            &addr_auth, APP_CONF.redis.host, APP_CONF.redis.port, APP_CONF.redis.database,
        );

        debug!("will connect to redis at: {}", tcp_addr_raw);

        let client = Client::open(tcp_addr_raw.as_ref()).expect("could not create redis client");

        info!("bound to store backend");

        CacheStore {
            client,
            timeout: Duration::from_secs(APP_CONF.redis.connection_timeout_seconds),
            connections: CacheStoreConnections {
                main: OnceCell::new(),
                scripts: OnceCell::new(),
            },
        }
    }
}

impl CacheStore {
    pub async fn get_meta(
        &self,
        shard: u8,
        key: String,
    ) -> Result<Option<(String, bool)>, CacheStoreError> {
        let mut connection = self.get_main_conn_unreliable().await?;

        match connection
            .hmget::<_, _, Vec<Value>>(&key, &[KEY_FINGERPRINT, KEY_COMPRESSED, KEY_TAGS])
            .await
        {
            Ok(values) => {
                let mut values_iter = values.into_iter();

                match (values_iter.next(), values_iter.next(), values_iter.next()) {
                    (
                        Some(Value::BulkString(fingerprint_bytes)),
                        Some(compressed_bytes),
                        Some(tags_bytes),
                    ) => {
                        // Parse compressed flag value (if any)
                        let compressed =
                            if let Value::BulkString(compressed_value) = compressed_bytes {
                                compressed_value == VALUE_COMPRESSED_YES
                            } else {
                                false
                            };

                        // Parse tags and bump their last access time
                        if let Value::BulkString(tags_bytes_data) = tags_bytes {
                            if let Ok(tags_data) = String::from_utf8(tags_bytes_data) {
                                if tags_data.is_empty() == false {
                                    let tags = tags_data
                                        .split(KEY_TAGS_SEPARATOR)
                                        .map(|tag| format!("{}:{}:{}", ROUTE_PREFIX, shard, tag))
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

                                    match redis::cmd("TOUCH")
                                        .arg(tags)
                                        .query_async::<usize>(&mut connection)
                                        .await
                                    {
                                        Ok(bump_count) => {
                                            // Partial bump count? Consider cache as non-existing
                                            if bump_count < tags_count {
                                                info!(
                                                    "got only partial tag count: {}/{}",
                                                    bump_count, tags_count
                                                );

                                                return Ok(None);
                                            }
                                        }
                                        Err(err) => {
                                            error!("error bumping access time of tags: {}", err);
                                        }
                                    }
                                }
                            }
                        }

                        // Decode raw bytes to string
                        if let Ok(fingerprint) = String::from_utf8(fingerprint_bytes) {
                            Ok(Some((fingerprint, compressed)))
                        } else {
                            Err(CacheStoreError::Corrupted)
                        }
                    }
                    (Some(Value::Nil), _, _) | (None, _, _) => Ok(None),
                    _ => Err(CacheStoreError::Invalid),
                }
            }
            _ => Err(CacheStoreError::Failed),
        }
    }

    pub async fn get_body(
        &self,
        key: String,
        compressed: bool,
    ) -> Result<Option<String>, CacheStoreError> {
        let mut connection = self.get_main_conn_unreliable().await?;

        match connection.hget::<_, _, Value>(&key, KEY_BODY).await {
            Ok(value) => match value {
                Value::BulkString(body_bytes_raw) => {
                    let body_bytes_result = if compressed {
                        // Decompress raw bytes
                        match zstd::decode_all(&body_bytes_raw[..]) {
                            Ok(decompress_bytes) => {
                                if body_bytes_raw.len() > 0 && decompress_bytes.len() == 0 {
                                    error!("decompressed store value has empty body");

                                    Err(())
                                } else {
                                    debug!(
                                        "decompressed store value from {} bytes to {} bytes",
                                        body_bytes_raw.len(),
                                        decompress_bytes.len()
                                    );

                                    Ok(decompress_bytes)
                                }
                            }
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
                }
                Value::Nil => Ok(None),
                _ => Err(CacheStoreError::Invalid),
            },
            _ => Err(CacheStoreError::Failed),
        }
    }

    pub async fn set(
        &self,
        key: String,
        key_mask: String,
        value: String,
        fingerprint: String,
        ttl: usize,
        key_tags: Vec<(String, String)>,
    ) -> CacheWriteResult {
        let body_size = value.len();

        // Cap TTL to 'max_key_expiration'
        let ttl_cap = cmp::min(ttl, APP_CONF.redis.max_key_expiration);

        // Ensure value is not larger than 'max_key_size'
        if body_size > APP_CONF.redis.max_key_size {
            return Err((CacheStoreError::TooLarge, fingerprint));
        }

        // Check if we should compress the body
        let compress_body =
            APP_CONF.cache.compress_body && body_size >= APP_CONF.cache.compress_above_bytes;

        // Compress value?
        let store_value_bytes_result = if compress_body == true {
            zstd::encode_all(value.as_bytes(), APP_CONF.cache.compress_level)
        } else {
            Ok(value.into_bytes())
        };

        let store_value_bytes = match store_value_bytes_result {
            Ok(bytes) => {
                if compress_body == true {
                    debug!(
                        "compressed store value from {} bytes to {} bytes",
                        body_size,
                        bytes.len()
                    );
                }

                bytes
            }
            Err(_) => {
                error!("error generating store value");

                return Err((CacheStoreError::Failed, fingerprint));
            }
        };

        // Generate compress value
        let compress_value_bytes = if compress_body {
            VALUE_COMPRESSED_YES
        } else {
            VALUE_COMPRESSED_NO
        };

        let mut pipeline = redis::pipe();

        // Append storage command
        {
            let key_tag_masks = key_tags
                .iter()
                .map(|key_tag| key_tag.1.as_ref())
                .collect::<Vec<&str>>();

            pipeline
                .hset_multiple(
                    &key,
                    &[
                        (KEY_FINGERPRINT, fingerprint.as_bytes()),
                        (KEY_TAGS, key_tag_masks.join(KEY_TAGS_SEPARATOR).as_bytes()),
                        (KEY_COMPRESSED, compress_value_bytes),
                        (KEY_BODY, &store_value_bytes),
                    ],
                )
                .ignore();
        }

        pipeline.expire(&key, ttl_cap as i64).ignore();

        for key_tag in &key_tags {
            pipeline.sadd(&key_tag.0, &key_mask).ignore();
            pipeline.expire(&key_tag.0, APP_CONF.redis.max_key_expiration as i64);
        }

        match self.get_main_conn_unreliable().await {
            Ok(mut connection) => match pipeline.query_async::<()>(&mut connection).await {
                Ok(_) => Ok(fingerprint),
                Err(err) => {
                    error!("got store error: {}", err);

                    Err((CacheStoreError::Failed, fingerprint))
                }
            },
            Err(err) => Err((err, fingerprint)),
        }
    }

    pub async fn purge_tag(
        &self,
        variant: &CachePurgeVariant,
        shard: u8,
        key_tag: &str,
    ) -> CachePurgeResult {
        let mut connection = self.get_scripts_conn().await?;

        let script_result = redis::Script::new(variant.get_script())
            .arg(ROUTE_PREFIX)
            .arg(shard)
            .arg(key_tag)
            .invoke_async::<()>(&mut connection)
            .await;

        script_result.or(Err(CacheStoreError::Failed))
    }

    async fn get_main_conn_unreliable(&self) -> Result<ConnectionManager, CacheStoreError> {
        // In the event of a Redis failure, 'get_main_conn_unreliable' allows \
        //   a full pass-through to be performed, thus ensuring service \
        //   continuity with degraded performance. If a reliable pool was \
        //   used there, then if the Redis server was down then it would block \
        //   for quite some time, meaning it would disrupt Bloom proxying \
        //   service (we need to be able to run in DIRECT mode with no cache \
        //   if Redis is down).
        self.connections
            .main
            .get_or_try_init(|| async {
                debug!(
                    "attempting to initialize main redis connection manager (unreliable mode)..."
                );

                // Important: entirely disable the retry algorithm, otherwise \
                //   a down Redis server will cause ingress HTTP connections \
                //   to wait for a very long time, until all retry attempts \
                //   have been exhausted.
                let config = redis::aio::ConnectionManagerConfig::new()
                    .set_connection_timeout(Some(self.timeout))
                    .set_response_timeout(Some(self.timeout))
                    .set_number_of_retries(0);

                match ConnectionManager::new_lazy_with_config(self.client.clone(), config) {
                    Ok(connection) => {
                        debug!("initialized main redis connection manager (unreliable mode)");

                        Ok(connection)
                    }
                    Err(err) => {
                        error!(
                            "could not create main redis connection manager: {} (unreliable mode)",
                            err
                        );

                        Err(CacheStoreError::Disconnected)
                    }
                }
            })
            .await
            .map(|connection| connection.clone())
    }

    async fn get_scripts_conn(&self) -> Result<ConnectionManager, CacheStoreError> {
        // 'get' is used as an alternative to 'try_get', when there is no \
        //   choice but to ensure an operation succeeds, even if it means \
        //   blocking for some time until the Redis server is available (eg. \
        //   for cache purges).
        self.connections
            .scripts
            .get_or_try_init(|| async {
                debug!("attempting to initialize scripts redis connection manager...");

                // Notice: configure the connection manager to retry \
                //   connecting to the Redis server if it is down. This will \
                //   block commands issued through this connection for some \
                //   time if the server is down, and only error out if all \
                //   attempts have been exhausted.
                let config = redis::aio::ConnectionManagerConfig::new()
                    .set_connection_timeout(Some(self.timeout))
                    .set_response_timeout(Some(self.timeout))
                    .set_number_of_retries(3);

                match ConnectionManager::new_lazy_with_config(self.client.clone(), config) {
                    Ok(connection) => {
                        debug!("initialized scripts redis connection manager");

                        Ok(connection)
                    }
                    Err(err) => {
                        error!("could not create scripts redis connection manager: {}", err);

                        Err(CacheStoreError::Disconnected)
                    }
                }
            })
            .await
            .map(|connection| connection.clone())
    }
}

impl CachePurgeVariant {
    fn get_script(&self) -> &'static str {
        // Notice: there is a limit of 1000 purgeable tags per bucket. Purging a lot of tags at \
        //   once is dangerous for Bloom, as the underlying Redis server is at risk of blocking.
        match *self {
            CachePurgeVariant::Bucket | CachePurgeVariant::Auth => {
                r#"
                    local count = redis.call('SCARD', ARGV[3])
                    local targets = {}

                    if count <= 1000 then
                        for _, tag in pairs(redis.call('SMEMBERS', ARGV[3])) do
                            table.insert(targets, ARGV[1] .. ":" .. ARGV[2] .. ":c:" .. tag)
                        end
                    end

                    table.insert(targets, ARGV[3])

                    redis.call('DEL', unpack(targets))
                "#
            }
        }
    }
}
