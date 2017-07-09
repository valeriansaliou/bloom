// Bloom
//
// HTTP REST API caching middleware
// Copyright: 2017, Valerian Saliou <valerian@valeriansaliou.name>
// License: Mozilla Public License v2.0 (MPL v2.0)

use ::APP_CACHE_STORE;

pub struct CacheRead;

impl CacheRead {
    pub fn acquire(key: &str) -> Result<String, &'static str> {
        match APP_CACHE_STORE.get(key) {
            Ok(Some(result)) => {
                Ok(result)
            }
            Ok(None) => {
                warn!("acquired empty value from cache for key: {}", key);

                Err("empty")
            }
            Err(err) => {
                error!("could not acquire value from cache for key: {} \
                    because: {}", key, err);

                Err(err)
            }
        }
    }
}
