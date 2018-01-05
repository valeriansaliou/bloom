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

type CacheReadResult = Result<(String, String), CacheReadError>;
type CacheReadFuture = Box<Future<Item = CacheReadResult, Error = ()>>;

impl CacheRead {
    pub fn acquire(shard: u8, key: &str, method: &Method) -> CacheReadFuture {
        if APP_CONF.cache.disable_read == false && CacheCheck::from_request(&method) == true {
            debug!("key: {} cacheable, reading cache", &key);

            Box::new(
                APP_CACHE_STORE
                    .get(shard, key.to_string())
                    .and_then(|acquired| if let Some(result) = acquired {
                        future::ok(Ok(result))
                    } else {
                        info!("acquired empty value from cache");

                        future::ok(Err(CacheReadError::Empty))
                    })
                    .or_else(|err| {
                        error!("could not acquire value from cache because: {:?}", err);

                        future::ok(Err(CacheReadError::StoreFailure))
                    }),
            )
        } else {
            debug!("key: {} not cacheable, ignoring (will pass through)", &key);

            Box::new(future::ok(Err(CacheReadError::PassThrough)))
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    #[should_panic]
    fn it_fails_acquiring_cache() {
        assert!(
            CacheRead::acquire("bloom:0:c:90d52bc6:f773d6f1", &Method::Get)
                .poll()
                .is_err()
        );
    }
}
