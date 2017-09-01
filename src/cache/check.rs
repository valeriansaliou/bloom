// Bloom
//
// HTTP REST API caching middleware
// Copyright: 2017, Valerian Saliou <valerian@valeriansaliou.name>
// License: Mozilla Public License v2.0 (MPL v2.0)

use hyper::{Method, StatusCode, Headers};

use header::response_ignore::HeaderResponseBloomResponseIgnore;

pub struct CacheCheck;

impl CacheCheck {
    pub fn from_request(method: &Method) -> bool {
        Self::is_cacheable_method(method) == true
    }

    pub fn from_response(method: &Method, status: &StatusCode, headers: &Headers) -> bool {
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
            StatusCode::UnsupportedMediaType |
            StatusCode::RangeNotSatisfiable |
            StatusCode::ExpectationFailed |
            StatusCode::ImATeapot |
            StatusCode::Locked |
            StatusCode::FailedDependency |
            StatusCode::PreconditionRequired |
            StatusCode::RequestHeaderFieldsTooLarge |
            StatusCode::NotImplemented |
            StatusCode::NotExtended => true,
            _ => false,
        }
    }

    fn is_cacheable_response(headers: &Headers) -> bool {
        // Ignore responses with 'Bloom-Response-Ignore'
        headers.has::<HeaderResponseBloomResponseIgnore>() == false
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn it_asserts_valid_cacheable_method() {
        assert_eq!(CacheCheck::is_cacheable_method(&Method::Get), true, "GET");
        assert_eq!(CacheCheck::is_cacheable_method(&Method::Head), true, "HEAD");
        assert_eq!(
            CacheCheck::is_cacheable_method(&Method::Options),
            false,
            "OPTIONS"
        );
        assert_eq!(
            CacheCheck::is_cacheable_method(&Method::Post),
            false,
            "POST"
        );
    }

    #[test]
    fn it_asserts_valid_cacheable_status() {
        assert_eq!(
            CacheCheck::is_cacheable_status(&StatusCode::Ok),
            true,
            "200 OK"
        );
        assert_eq!(
            CacheCheck::is_cacheable_status(&StatusCode::Unauthorized),
            true,
            "401 OK"
        );
        assert_eq!(
            CacheCheck::is_cacheable_status(&StatusCode::BadRequest),
            false,
            "400 Bad Request"
        );
        assert_eq!(
            CacheCheck::is_cacheable_status(&StatusCode::InternalServerError),
            false,
            "500 Internal Server Error"
        );
    }
}
