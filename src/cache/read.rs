// Bloom
//
// HTTP REST API caching middleware
// Copyright: 2017, Valerian Saliou <valerian@valeriansaliou.name>
// License: Mozilla Public License v2.0 (MPL v2.0)

use hyper::Method;

use super::check::CacheCheck;

use APP_CONF;
use APP_CACHE_STORE;

pub struct CacheRead;

pub enum CacheReadError {
    PassThrough,
    Empty,
    StoreFailure,
}

impl CacheRead {
    pub fn acquire(key: &str, method: &Method) -> Result<String, CacheReadError> {
        if APP_CONF.cache.disable_read == false && CacheCheck::from_request(method) == true {
            debug!("key: {} cacheable, reading cache", key);

            match APP_CACHE_STORE.get(key) {
                Ok(Some(result)) => Ok(result),
                Ok(None) => {
                    info!("acquired empty value from cache for key: {}", key);

                    Err(CacheReadError::Empty)
                }
                Err(err) => {
                    error!(
                        "could not acquire value from cache for key: {} because: {:?}",
                        key,
                        err
                    );

                    Err(CacheReadError::StoreFailure)
                }
            }
        } else {
            debug!("key: {} not cacheable, ignoring (will pass through)", key);

            Err(CacheReadError::PassThrough)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    #[should_panic]
    fn it_fails_acquiring_cache() {
        assert!(CacheRead::acquire("bloom:0:90d52bc6:f773d6f1", &Method::Get).is_err());
    }
}
