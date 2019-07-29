// Bloom
//
// HTTP REST API caching middleware
// Copyright: 2017, Valerian Saliou <valerian@valeriansaliou.name>
// License: Mozilla Public License v2.0 (MPL v2.0)

macro_rules! get_cache_store_client {
    ($pool:expr, $error:expr, $client:ident $code:block) => {
        // In the event of a Redis failure, 'try_get' allows a full pass-through to be performed, \
        //   thus ensuring service continuity with degraded performance. If 'get' was used there, \
        //   performing as many pool 'get' as the pool size would incur a synchronous locking, \
        //   which would wait forever until the Redis connection is restored (this is dangerous).
        match $pool.try_get() {
            Some(mut $client) => $code,
            None => {
                error!("failed getting a cache store client from pool");

                Err($error)
            }
        }
    };
}
