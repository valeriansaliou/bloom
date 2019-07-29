// Bloom
//
// HTTP REST API caching middleware
// Copyright: 2017, Valerian Saliou <valerian@valeriansaliou.name>
// License: Mozilla Public License v2.0 (MPL v2.0)

macro_rules! get_cache_store_client_try {
    ($pool:expr, $error:expr, $client:ident $code:block) => {
        // In the event of a Redis failure, 'try_get' allows a full pass-through to be performed, \
        //   thus ensuring service continuity with degraded performance. If 'get' was used there, \
        //   performing as many pool 'get' as the pool size would incur a synchronous locking, \
        //   which would wait forever until the Redis connection is restored (this is dangerous).
        match $pool.try_get() {
            Some(mut $client) => $code,
            None => {
                error!("failed getting a cache store client from pool (try mode)");

                Err($error)
            }
        }
    };
}

macro_rules! get_cache_store_client_wait {
    ($pool:expr, $error:expr, $client:ident $code:block) => {
        // 'get' is used as an alternative to 'try_get', when there is no choice but to ensure \
        //   an operation succeeds (eg. for cache purges).
        match $pool.get() {
            Ok(mut $client) => $code,
            Err(err) => {
                error!(
                    "failed getting a cache store client from pool (wait mode), because: {}",
                    err
                );

                Err($error)
            }
        }
    };
}
