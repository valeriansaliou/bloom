// Bloom
//
// HTTP REST API caching middleware
// Copyright: 2017, Valerian Saliou <valerian@valeriansaliou.name>
// License: Mozilla Public License v2.0 (MPL v2.0)

use std::str;
use hyper::{Method, HttpVersion, StatusCode, Headers, Body};
use futures::{Future, Stream};

use super::route::CacheRoute;
use APP_CONF;
use APP_CACHE_STORE;
use header::janitor::HeaderJanitor;
use header::response_ignore::HeaderResponseBloomResponseIgnore;
use header::response_bucket::HeaderResponseBloomResponseBucket;
use header::response_ttl::HeaderResponseBloomResponseTTL;

pub struct CacheWrite;

pub struct CacheWriteResult {
    pub body: Result<String, Option<String>>,
    pub value: Option<String>,
}

impl CacheWrite {
    pub fn save(
        key: &str,
        method: &Method,
        version: &HttpVersion,
        status: &StatusCode,
        headers: &Headers,
        body: Body,
    ) -> CacheWriteResult {
        // TODO: wait() is unsafe for event loop, see: \
        // https://docs.rs/futures/0.1.14/futures/stream/trait.Stream.html\
        //   #method.wait
        // FIX: green-threads?

        // TODO: fix next() chunk infinite wait (if no. chunk > 1)

        let body_result = body.map_err(|_| ())
            .concat2()
            .and_then(|chunk| String::from_utf8(chunk.to_vec()).map_err(|_| ()))
            .wait();

        match body_result {
            Ok(body_value) => {
                debug!("checking whether to write cache for key: {}", key);

                if Self::is_cacheable(method, status, headers) == true {
                    debug!("key: {} cacheable, writing cache", key);

                    // Acquire bucket from response, or fallback to no bucket
                    let key_bucket = match headers.get::<HeaderResponseBloomResponseBucket>() {
                        None => None,
                        Some(value) => {
                            Some(CacheRoute::gen_key_bucket_with_ns(key,
                                &CacheRoute::hash(&value.0)))
                        },
                    };

                    // Acquire TTL from response, or fallback to default TTL
                    let ttl = match headers.get::<HeaderResponseBloomResponseTTL>() {
                        None => APP_CONF.cache.ttl_default,
                        Some(value) => value.0,
                    };

                    // Generate storable value
                    let value = format!(
                        "{}\n{}\n{}",
                        CacheWrite::generate_chain_banner(version, status),
                        CacheWrite::generate_chain_headers(headers),
                        body_value
                    );

                    // Write to cache
                    match APP_CACHE_STORE.set(key, &value, ttl, key_bucket).wait() {
                        Ok(_) => {
                            debug!("wrote cache for key: {}", key);

                            CacheWriteResult {
                                body: Ok(body_value),
                                value: Some(value),
                            }
                        }
                        Err(err) => {
                            warn!(
                                "could not write cache for key: {} \
                                    because: {:?}",
                                key,
                                err
                            );

                            CacheWriteResult {
                                body: Err(Some(body_value)),
                                value: Some(value),
                            }
                        }
                    }
                } else {
                    debug!("key: {} not cacheable, ignoring", key);

                    // Not cacheable, ignore
                    CacheWriteResult {
                        body: Err(Some(body_value)),
                        value: None,
                    }
                }
            }
            _ => {
                error!("failed unwrapping body value for key: {}, ignoring", key);

                CacheWriteResult {
                    body: Err(None),
                    value: None,
                }
            }
        }
    }

    fn is_cacheable(method: &Method, status: &StatusCode, headers: &Headers) -> bool {
        Self::is_cacheable_method(method) == true && Self::is_cacheable_status(status) == true &&
            Self::is_cacheable_response(headers) == true
    }

    fn is_cacheable_method(method: &Method) -> bool {
        match *method {
            Method::Get | Method::Head => true,
            _ => false,
        }
    }

    fn is_cacheable_status(status: &StatusCode) -> bool {
        match *status {
            StatusCode::Ok |
            StatusCode::NonAuthoritativeInformation |
            StatusCode::NoContent |
            StatusCode::ResetContent |
            StatusCode::PartialContent |
            StatusCode::MultiStatus |
            StatusCode::AlreadyReported |
            StatusCode::MultipleChoices |
            StatusCode::MovedPermanently |
            StatusCode::Found |
            StatusCode::SeeOther |
            StatusCode::PermanentRedirect |
            StatusCode::Unauthorized |
            StatusCode::PaymentRequired |
            StatusCode::Forbidden |
            StatusCode::NotFound |
            StatusCode::MethodNotAllowed |
            StatusCode::Gone |
            StatusCode::UriTooLong |
            StatusCode::Locked |
            StatusCode::FailedDependency |
            StatusCode::NotImplemented => true,
            _ => false,
        }
    }

    fn is_cacheable_response(headers: &Headers) -> bool {
        // Ignore responses with 'Bloom-Response-Ignore'
        headers.has::<HeaderResponseBloomResponseIgnore>() == false
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
                "bloom:0:90d52bc6:f773d6f1",
                &Method::Get,
                &HttpVersion::Http11,
                &StatusCode::Ok,
                &Headers::new(),
                Body::empty(),
            ).body
                .is_err()
        );
    }

    #[test]
    fn it_asserts_valid_cacheable_method() {
        assert_eq!(CacheWrite::is_cacheable_method(&Method::Get), true, "GET");
        assert_eq!(CacheWrite::is_cacheable_method(&Method::Head), true, "HEAD");
        assert_eq!(
            CacheWrite::is_cacheable_method(&Method::Options),
            false,
            "OPTIONS"
        );
        assert_eq!(
            CacheWrite::is_cacheable_method(&Method::Post),
            false,
            "POST"
        );
    }

    #[test]
    fn it_asserts_valid_cacheable_status() {
        assert_eq!(
            CacheWrite::is_cacheable_status(&StatusCode::Ok),
            true,
            "200 OK"
        );
        assert_eq!(
            CacheWrite::is_cacheable_status(&StatusCode::Unauthorized),
            true,
            "401 OK"
        );
        assert_eq!(
            CacheWrite::is_cacheable_status(&StatusCode::BadRequest),
            false,
            "400 Bad Request"
        );
        assert_eq!(
            CacheWrite::is_cacheable_status(&StatusCode::InternalServerError),
            false,
            "500 Internal Server Error"
        );
    }
}
