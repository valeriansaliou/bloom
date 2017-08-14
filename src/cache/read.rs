// Bloom
//
// HTTP REST API caching middleware
// Copyright: 2017, Valerian Saliou <valerian@valeriansaliou.name>
// License: Mozilla Public License v2.0 (MPL v2.0)

use futures::Future;

use APP_CACHE_STORE;

pub struct CacheRead;

pub enum CacheReadError {
    Empty,
    StoreFailure,
}

impl CacheRead {
    pub fn acquire(key: &str) -> Result<String, CacheReadError> {
        match APP_CACHE_STORE.get(key).wait() {
            Ok(Some(result)) => Ok(result),
            Ok(None) => {
                warn!("acquired empty value from cache for key: {}", key);

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
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    #[should_panic]
    fn it_fails_acquiring_cache() {
        assert!(CacheRead::acquire("bloom:0:90d52bc6:f773d6f1").is_err());
    }
}
