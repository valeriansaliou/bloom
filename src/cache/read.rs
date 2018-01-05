// Bloom
//
// HTTP REST API caching middleware
// Copyright: 2017, Valerian Saliou <valerian@valeriansaliou.name>
// License: Mozilla Public License v2.0 (MPL v2.0)

use hyper::Method;
use futures::future::{self, Future};

use super::check::CacheCheck;

use APP_CONF;
use APP_CACHE_STORE;

pub struct CacheRead;

pub enum CacheReadError {
    PassThrough,
    Empty,
    StoreFailure,
}

type CacheReadResult = Result<String, CacheReadError>;
type CacheReadResultFuture = Box<Future<Item = CacheReadResult, Error = ()>>;

type CacheReadOptionalResult = Result<Option<String>, CacheReadError>;
type CacheReadOptionalResultFuture = Box<Future<Item = CacheReadOptionalResult, Error = ()>>;

impl CacheRead {
    pub fn acquire_meta(shard: u8, key: &str, method: &Method) -> CacheReadResultFuture {
        if APP_CONF.cache.disable_read == false && CacheCheck::from_request(&method) == true {
            debug!("key: {} cacheable, reading cache", &key);

            Box::new(
                APP_CACHE_STORE
                    .get_meta(shard, key.to_string())
                    .and_then(|acquired| if let Some(result) = acquired {
                        future::ok(Ok(result))
                    } else {
                        info!("acquired empty meta value from cache");

                        future::ok(Err(CacheReadError::Empty))
                    })
                    .or_else(|err| {
                        error!("could not acquire meta value from cache because: {:?}", err);

                        future::ok(Err(CacheReadError::StoreFailure))
                    }),
            )
        } else {
            debug!("key: {} not cacheable, ignoring (will pass through)", &key);

            Box::new(future::ok(Err(CacheReadError::PassThrough)))
        }
    }

    pub fn acquire_body(key: &str) -> CacheReadOptionalResultFuture {
        Box::new(
            APP_CACHE_STORE
                .get_body(key.to_string())
                .and_then(|acquired| if let Some(result) = acquired {
                    future::ok(Ok(Some(result)))
                } else {
                    info!("acquired empty body value from cache");

                    future::ok(Err(CacheReadError::Empty))
                })
                .or_else(|err| {
                    error!("could not acquire body value from cache because: {:?}", err);

                    future::ok(Err(CacheReadError::StoreFailure))
                }),
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    #[should_panic]
    fn it_fails_acquiring_cache_meta() {
        assert!(
            CacheRead::acquire_meta(0, "bloom:0:c:90d52bc6:f773d6f1", &Method::Get)
                .poll()
                .is_err()
        );
    }

    #[test]
    #[should_panic]
    fn it_fails_acquiring_cache_body() {
        assert!(
            CacheRead::acquire_body("bloom:0:c:90d52bc6:f773d6f1")
                .poll()
                .is_err()
        );
    }
}
