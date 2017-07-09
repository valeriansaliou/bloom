// Bloom
//
// HTTP REST API caching middleware
// Copyright: 2017, Valerian Saliou <valerian@valeriansaliou.name>
// License: Mozilla Public License v2.0 (MPL v2.0)

use std::str;
use hyper::{Method, StatusCode, Headers, Body};
use hyper::server::{Request, Response};

use ::APP_CONF;
use ::APP_CACHE_STORE;
use header::janitor::HeaderJanitor;
use header::response_ignore::HeaderResponseBloomResponseIgnore;
use header::response_ttl::HeaderResponseBloomResponseTTL;

pub struct CacheWrite;

impl CacheWrite {
    pub fn save(key: &str, req: Request, res: Response) -> bool {
        let method = req.method();
        let status = res.status();
        let headers = res.headers();

        if Self::is_cacheable(&method, &status, &headers)
            == true {
            // Acquire TTL from response, or fallback to default TTL
            let ttl = match headers.get::<HeaderResponseBloomResponseTTL>() {
                None => APP_CONF.cache.ttl_default,
                Some(value) => value.0
            };

            // Generate storable value
            let value = format!("{}\n{}\n\n{}",
                CacheWrite::generate_chain_status(&status),
                CacheWrite::generate_chain_headers(&headers),
                CacheWrite::generate_chain_body(()));

            // Write to cache
            APP_CACHE_STORE.set(key, &value, ttl).is_ok()
        } else {
            // Not cacheable, ignore
            false
        }
    }

    fn is_cacheable(method: &Method, status: &StatusCode, headers: &Headers)
        -> bool {
        Self::is_cacheable_method(method) == true &&
            Self::is_cacheable_status(status) == true &&
            Self::is_cacheable_response(headers) == true
    }

    fn is_cacheable_method(method: &Method) -> bool {
        match *method {
            Method::Get | Method::Head => true,
            _ => false
        }
    }

    fn is_cacheable_status(status: &StatusCode) -> bool {
        match *status {
            StatusCode::Ok
            | StatusCode::NonAuthoritativeInformation
            | StatusCode::NoContent
            | StatusCode::ResetContent
            | StatusCode::PartialContent
            | StatusCode::MultiStatus
            | StatusCode::AlreadyReported
            | StatusCode::MultipleChoices
            | StatusCode::MovedPermanently
            | StatusCode::Found
            | StatusCode::SeeOther
            | StatusCode::PermanentRedirect
            | StatusCode::Unauthorized
            | StatusCode::PaymentRequired
            | StatusCode::Forbidden
            | StatusCode::NotFound
            | StatusCode::MethodNotAllowed
            | StatusCode::Gone
            | StatusCode::UriTooLong
            | StatusCode::Locked
            | StatusCode::FailedDependency
            | StatusCode::NotImplemented => {
                true
            }
            _ => {
                false
            }
        }
    }

    fn is_cacheable_response(headers: &Headers) -> bool {
        // Ignore responses with 'Bloom-Response-Ignore'
        headers.has::<HeaderResponseBloomResponseIgnore>()
    }

    fn generate_chain_status(status: &StatusCode) -> String {
        // TODO
        String::from("200")
    }

    fn generate_chain_headers(headers: &Headers) -> String {
        let mut chain_headers = String::new();

        for header in headers.iter() {
            // Ensure no contextual header is added to cache
            if HeaderJanitor::is_contextual(&header) == false {
                match header.raw().one() {
                    Some(header_raw) => {
                        match str::from_utf8(header_raw) {
                            Ok(header_str) => {
                                chain_headers.push_str(header_str)
                            }
                            _ => ()
                        }
                    }
                    _ => ()
                }
            }
        }

        chain_headers
    }

    fn generate_chain_body(body: ()) -> String {
        // TODO
        String::from("{}")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn it_asserts_valid_cacheable_method() {
        assert_eq!(CacheWrite::is_cacheable_method(&Method::Get),
            true, "GET");
        assert_eq!(CacheWrite::is_cacheable_method(&Method::Head),
            true, "HEAD");
        assert_eq!(CacheWrite::is_cacheable_method(&Method::Options),
            false, "OPTIONS");
        assert_eq!(CacheWrite::is_cacheable_method(&Method::Post),
            false, "POST");
    }

    #[test]
    fn it_asserts_valid_cacheable_status() {
        assert_eq!(CacheWrite::is_cacheable_status(&StatusCode::Ok),
            true, "200 OK");
        assert_eq!(CacheWrite::is_cacheable_status(&StatusCode::Unauthorized),
            true, "401 OK");
        assert_eq!(CacheWrite::is_cacheable_status(&StatusCode::BadRequest),
            false, "400 Bad Request");
        assert_eq!(CacheWrite::is_cacheable_status(&StatusCode::InternalServerError),
            false, "500 Internal Server Error");
    }
}
