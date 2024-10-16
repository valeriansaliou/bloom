// Bloom
//
// HTTP REST API caching middleware
// Copyright: 2017, Valerian Saliou <valerian@valeriansaliou.name>
// License: Mozilla Public License v2.0 (MPL v2.0)

use hyper::{Headers, Method, StatusCode};

use crate::header::response_ignore::HeaderResponseBloomResponseIgnore;

pub struct CacheCheck;

impl CacheCheck {
    pub const fn from_request(method: &Method) -> bool {
        Self::is_cacheable_method(method)
    }

    pub fn from_response(method: &Method, status: StatusCode, headers: &Headers) -> bool {
        Self::is_cacheable_method(method)
            && Self::is_cacheable_status(status)
            && Self::is_cacheable_response(headers)
    }

    const fn is_cacheable_method(method: &Method) -> bool {
        matches!(*method, Method::Get | Method::Head | Method::Options)
    }

    const fn is_cacheable_status(status: StatusCode) -> bool {
        matches!(
            status,
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
                | StatusCode::UnsupportedMediaType
                | StatusCode::RangeNotSatisfiable
                | StatusCode::ExpectationFailed
                | StatusCode::ImATeapot
                | StatusCode::Locked
                | StatusCode::FailedDependency
                | StatusCode::PreconditionRequired
                | StatusCode::RequestHeaderFieldsTooLarge
                | StatusCode::NotImplemented
                | StatusCode::NotExtended
        )
    }

    fn is_cacheable_response(headers: &Headers) -> bool {
        // Ignore responses with 'Bloom-Response-Ignore'
        !headers.has::<HeaderResponseBloomResponseIgnore>()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn it_asserts_valid_cacheable_method() {
        assert!(CacheCheck::is_cacheable_method(&Method::Get), "GET");
        assert!(CacheCheck::is_cacheable_method(&Method::Head), "HEAD");
        assert!(CacheCheck::is_cacheable_method(&Method::Options), "OPTIONS");
        assert!(!CacheCheck::is_cacheable_method(&Method::Post), "POST");
    }

    #[test]
    fn it_asserts_valid_cacheable_status() {
        assert!(CacheCheck::is_cacheable_status(StatusCode::Ok), "200 OK");
        assert!(
            CacheCheck::is_cacheable_status(StatusCode::Unauthorized),
            "401 OK"
        );
        assert!(
            !CacheCheck::is_cacheable_status(StatusCode::BadRequest),
            "400 Bad Request"
        );
        assert!(
            !CacheCheck::is_cacheable_status(StatusCode::InternalServerError),
            "500 Internal Server Error"
        );
    }
}
