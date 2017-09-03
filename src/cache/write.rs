// Bloom
//
// HTTP REST API caching middleware
// Copyright: 2017, Valerian Saliou <valerian@valeriansaliou.name>
// License: Mozilla Public License v2.0 (MPL v2.0)

use std::str;
use hyper::{Error, Method, HttpVersion, StatusCode, Headers, Body};
use futures::{future, Future, Stream};

use super::route::CacheRoute;
use super::check::CacheCheck;
use APP_CONF;
use APP_CACHE_STORE;
use header::janitor::HeaderJanitor;
use header::response_buckets::HeaderResponseBloomResponseBuckets;
use header::response_ttl::HeaderResponseBloomResponseTTL;

pub struct CacheWrite;

pub struct CacheWriteResult {
    pub body: Result<String, Option<String>>,
    pub value: Option<String>,
    pub status: StatusCode,
    pub headers: Headers,
}

pub type CacheWriteResultFuture = Box<Future<Item = CacheWriteResult, Error = Error>>;

impl CacheWrite {
    pub fn save(
        key: String,
        key_mask: String,
        auth_hash: String,
        shard: u8,
        method: Method,
        version: HttpVersion,
        status: StatusCode,
        mut headers: Headers,
        body: Body,
    ) -> CacheWriteResultFuture {
        Box::new(
            body.concat2()
                .map(|raw_data| String::from_utf8(raw_data.to_vec()))
                .and_then(move |body_result| {
                    future::ok(match body_result {
                        Ok(body_value) => {
                            debug!("checking whether to write cache for key: {}", &key);

                            if APP_CONF.cache.disable_write == false &&
                                CacheCheck::from_response(&method, &status, &headers) == true
                            {
                                debug!("key: {} cacheable, writing cache", &key);

                                // Acquire bucket from response, or fallback to no bucket
                                let mut key_tags =
                                    match headers.get::<HeaderResponseBloomResponseBuckets>() {
                                        None => Vec::new(),
                                        Some(value) => {
                                            value
                                                .0
                                                .iter()
                                                .map(|value| {
                                                    CacheRoute::gen_key_bucket_from_hash(
                                                        shard,
                                                        &CacheRoute::hash(value),
                                                    )
                                                })
                                                .collect::<Vec<String>>()
                                        }
                                    };

                                key_tags.push(CacheRoute::gen_key_auth_from_hash(
                                    shard,
                                    &auth_hash,
                                ));

                                // Acquire TTL from response, or fallback to default TTL
                                let ttl = match headers.get::<HeaderResponseBloomResponseTTL>() {
                                    None => APP_CONF.cache.ttl_default,
                                    Some(value) => value.0,
                                };

                                // Clean headers before they get stored
                                HeaderJanitor::clean(&mut headers);

                                // Generate storable value
                                let value = format!(
                                    "{}\n{}\n{}",
                                    CacheWrite::generate_chain_banner(&version, &status),
                                    CacheWrite::generate_chain_headers(&headers),
                                    body_value
                                );

                                // Write to cache
                                match APP_CACHE_STORE.set(&key, &key_mask, &value, ttl, key_tags) {
                                    Ok(_) => {
                                        debug!("wrote cache for key: {}", &key);

                                        CacheWriteResult {
                                            body: Ok(body_value),
                                            value: Some(value),
                                            status: status,
                                            headers: headers,
                                        }
                                    }
                                    Err(err) => {
                                        warn!(
                                            "could not write cache for key: {} because: {:?}",
                                            &key,
                                            err
                                        );

                                        CacheWriteResult {
                                            body: Err(Some(body_value)),
                                            value: Some(value),
                                            status: status,
                                            headers: headers,
                                        }
                                    }
                                }
                            } else {
                                debug!("key: {} not cacheable, ignoring", &key);

                                // Not cacheable, ignore
                                CacheWriteResult {
                                    body: Err(Some(body_value)),
                                    value: None,
                                    status: status,
                                    headers: headers,
                                }
                            }
                        }
                        _ => {
                            error!("failed unwrapping body value for key: {}, ignoring", &key);

                            CacheWriteResult {
                                body: Err(None),
                                value: None,
                                status: status,
                                headers: headers,
                            }
                        }
                    })
                }),
        )
    }

    fn generate_chain_banner(version: &HttpVersion, status: &StatusCode) -> String {
        format!("{} {}", version, status)
    }

    fn generate_chain_headers(headers: &Headers) -> String {
        headers
            .iter()
            .filter(|header_view| {
                HeaderJanitor::is_contextual(&header_view) == false
            })
            .map(|header_view| {
                format!("{}: {}\n", header_view.name(), header_view.value_string())
            })
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    #[should_panic]
    fn it_fails_saving_cache() {
        assert!(
            CacheWrite::save(
                "bloom:0:c:90d52bc6:f773d6f1".to_string(),
                "90d52bc6:f773d6f1".to_string(),
                "90d52bc6".to_string(),
                0,
                Method::Get,
                HttpVersion::Http11,
                StatusCode::Ok,
                Headers::new(),
                Body::empty(),
            ).poll()
                .is_err()
        );
    }
}
