// Bloom
//
// HTTP REST API caching middleware
// Copyright: 2017, Valerian Saliou <valerian@valeriansaliou.name>
// License: Mozilla Public License v2.0 (MPL v2.0)

use hyper::Method;

use super::check::CacheCheck;

use crate::APP_CACHE_STORE;
use crate::APP_CONF;

pub struct CacheRead;

pub enum CacheReadError {
    PassThrough,
    Empty,
    StoreFailure,
}

type CacheReadResult = Result<(String, bool), CacheReadError>;
type CacheReadOptionalResult = Result<Option<String>, CacheReadError>;

impl CacheRead {
    pub async fn acquire_meta(
        shard: u8,
        key: &str,
        method: &Method,
    ) -> Result<CacheReadResult, ()> {
        if APP_CONF.cache.disable_read == false && CacheCheck::from_request(method) == true {
            debug!("key: {} cacheable, reading cache", &key);

            match APP_CACHE_STORE.get_meta(shard, key.to_string()).await {
                Ok(Some(result)) => Ok(Ok(result)),
                Ok(None) => {
                    info!("acquired empty meta value from cache");

                    Ok(Err(CacheReadError::Empty))
                }
                Err(err) => {
                    error!("could not acquire meta value from cache because: {:?}", err);

                    Ok(Err(CacheReadError::StoreFailure))
                }
            }
        } else {
            debug!("key: {} not cacheable, ignoring (will pass through)", &key);

            Ok(Err(CacheReadError::PassThrough))
        }
    }

    pub async fn acquire_body(key: &str, compressed: bool) -> Result<CacheReadOptionalResult, ()> {
        match APP_CACHE_STORE.get_body(key.to_string(), compressed).await {
            Ok(Some(result)) => Ok(Ok(Some(result))),
            Ok(None) => {
                info!("acquired empty body value from cache");

                Ok(Err(CacheReadError::Empty))
            }
            Err(err) => {
                error!("could not acquire body value from cache because: {:?}", err);

                Ok(Err(CacheReadError::StoreFailure))
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    #[should_panic]
    async fn it_fails_acquiring_cache_meta() {
        assert!(
            CacheRead::acquire_meta(0, "bloom:0:c:90d52bc6:f773d6f1", &Method::GET)
                .await
                .is_err()
        );
    }

    #[tokio::test]
    #[should_panic]
    async fn it_fails_acquiring_cache_body() {
        assert!(
            CacheRead::acquire_body("bloom:0:c:90d52bc6:f773d6f1", false)
                .await
                .is_err()
        );
    }
}
