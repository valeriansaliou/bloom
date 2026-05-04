// Bloom
//
// HTTP REST API caching middleware
// Copyright: 2017, Valerian Saliou <valerian@valeriansaliou.name>
// License: Mozilla Public License v2.0 (MPL v2.0)

use http::Method;

use super::check::CacheCheck;

use crate::APP_CACHE_STORE;
use crate::APP_CONF;

pub struct CacheRead;

#[derive(Debug)]
pub enum CacheReadError {
    PassThrough,
    Empty,
    StoreFailure,
}

pub type CacheReadResult = Result<String, CacheReadError>;
pub type CacheReadOptionalResult = Result<Option<String>, CacheReadError>;

impl CacheRead {
    pub async fn acquire_meta(shard: u8, key: &str, method: &Method) -> CacheReadResult {
        if APP_CONF.cache.disable_read == false && CacheCheck::from_request(&method) == true {
            debug!("key: {} cacheable, reading cache", &key);

            match APP_CACHE_STORE.get_meta(shard, key.to_string()).await {
                Ok(Some(result)) => Ok(result),
                Ok(None) => {
                    info!("acquired empty meta value from cache");
                    Err(CacheReadError::Empty)
                }
                Err(err) => {
                    error!("could not acquire meta value from cache because: {:?}", err);
                    Err(CacheReadError::StoreFailure)
                }
            }
        } else {
            debug!("key: {} not cacheable, ignoring (will pass through)", &key);
            Err(CacheReadError::PassThrough)
        }
    }

    pub async fn acquire_body(key: &str) -> CacheReadOptionalResult {
        match APP_CACHE_STORE.get_body(key.to_string()).await {
            Ok(Some(result)) => Ok(Some(result)),
            Ok(None) => {
                info!("acquired empty body value from cache");
                Err(CacheReadError::Empty)
            }
            Err(err) => {
                error!("could not acquire body value from cache because: {:?}", err);
                Err(CacheReadError::StoreFailure)
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn it_returns_empty_for_missing_cache_meta() {
        let result = CacheRead::acquire_meta(0, "bloom:0:c:test:nonexistent", &Method::GET).await;
        assert!(
            matches!(result, Err(CacheReadError::Empty) | Err(CacheReadError::StoreFailure)),
            "Expected Empty or StoreFailure error for non-existent key"
        );
    }

    #[tokio::test]
    async fn it_returns_empty_for_missing_cache_body() {
        let result = CacheRead::acquire_body("bloom:0:c:test:nonexistent").await;
        assert!(
            matches!(result, Err(CacheReadError::Empty) | Err(CacheReadError::StoreFailure)),
            "Expected Empty or StoreFailure error for non-existent key"
        );
    }
}
