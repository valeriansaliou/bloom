// Bloom
//
// HTTP REST API caching middleware
// Copyright: 2017, Valerian Saliou <valerian@valeriansaliou.name>
// License: Mozilla Public License v2.0 (MPL v2.0)

use http::header::HeaderMap;
use http::{Method, StatusCode};

use crate::header::response_ignore;

pub struct CacheCheck;

impl CacheCheck {
    pub fn from_request(method: &Method) -> bool {
        Self::is_cacheable_method(method) == true
    }

    pub fn from_response(method: &Method, status: &StatusCode, headers: &HeaderMap) -> bool {
        Self::is_cacheable_method(method) == true
            && Self::is_cacheable_status(status) == true
            && Self::is_cacheable_response(headers) == true
    }

    fn is_cacheable_method(method: &Method) -> bool {
        matches!(method, &Method::GET | &Method::HEAD | &Method::OPTIONS)
    }

    fn is_cacheable_status(status: &StatusCode) -> bool {
        matches!(
            *status,
            StatusCode::OK
                | StatusCode::NON_AUTHORITATIVE_INFORMATION
                | StatusCode::NO_CONTENT
                | StatusCode::RESET_CONTENT
                | StatusCode::PARTIAL_CONTENT
                | StatusCode::MULTI_STATUS
                | StatusCode::ALREADY_REPORTED
                | StatusCode::MULTIPLE_CHOICES
                | StatusCode::MOVED_PERMANENTLY
                | StatusCode::FOUND
                | StatusCode::SEE_OTHER
                | StatusCode::PERMANENT_REDIRECT
                | StatusCode::UNAUTHORIZED
                | StatusCode::PAYMENT_REQUIRED
                | StatusCode::FORBIDDEN
                | StatusCode::NOT_FOUND
                | StatusCode::METHOD_NOT_ALLOWED
                | StatusCode::GONE
                | StatusCode::URI_TOO_LONG
                | StatusCode::UNSUPPORTED_MEDIA_TYPE
                | StatusCode::RANGE_NOT_SATISFIABLE
                | StatusCode::EXPECTATION_FAILED
                | StatusCode::IM_A_TEAPOT
                | StatusCode::LOCKED
                | StatusCode::FAILED_DEPENDENCY
                | StatusCode::PRECONDITION_REQUIRED
                | StatusCode::REQUEST_HEADER_FIELDS_TOO_LARGE
                | StatusCode::NOT_IMPLEMENTED
                | StatusCode::NOT_EXTENDED
        )
    }

    fn is_cacheable_response(headers: &HeaderMap) -> bool {
        if let Some(value) = headers.get(response_ignore::HEADER_NAME) {
            if let Ok(v) = value.to_str() {
                return !response_ignore::should_ignore(v);
            }
        }
        true
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn it_asserts_valid_cacheable_method() {
        assert_eq!(CacheCheck::is_cacheable_method(&Method::GET), true, "GET");
        assert_eq!(
            CacheCheck::is_cacheable_method(&Method::HEAD),
            true,
            "HEAD"
        );
        assert_eq!(
            CacheCheck::is_cacheable_method(&Method::OPTIONS),
            true,
            "OPTIONS"
        );
        assert_eq!(
            CacheCheck::is_cacheable_method(&Method::POST),
            false,
            "POST"
        );
    }

    #[test]
    fn it_asserts_valid_cacheable_status() {
        assert_eq!(
            CacheCheck::is_cacheable_status(&StatusCode::OK),
            true,
            "200 OK"
        );
        assert_eq!(
            CacheCheck::is_cacheable_status(&StatusCode::UNAUTHORIZED),
            true,
            "401 OK"
        );
        assert_eq!(
            CacheCheck::is_cacheable_status(&StatusCode::BAD_REQUEST),
            false,
            "400 Bad Request"
        );
        assert_eq!(
            CacheCheck::is_cacheable_status(&StatusCode::INTERNAL_SERVER_ERROR),
            false,
            "500 Internal Server Error"
        );
    }
}
