// Bloom
//
// HTTP REST API caching middleware
// Copyright: 2017, Valerian Saliou <valerian@valeriansaliou.name>
// License: Mozilla Public License v2.0 (MPL v2.0)

macro_rules! get_cache_store_client {
    ($self:ident, $client:ident $code:block) => (
        future::result(
            match $self.pool.get() {
                Ok($client) => $code,
                _ => Err(CacheStoreError::Disconnected),
            }
        )
    )
}

macro_rules! gen_cache_store_empty_result {
    ($pattern:expr) => (
        match $pattern {
            Ok(_) => Ok(None),
            Err(err) => {
                error!("got store error: {}", err);

                Err(CacheStoreError::Failed)
            },
        }
    )
}
