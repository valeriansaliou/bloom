// Bloom
//
// HTTP REST API caching middleware
// Copyright: 2017, Valerian Saliou <valerian@valeriansaliou.name>
// License: Mozilla Public License v2.0 (MPL v2.0)

use hyper::{Method, StatusCode};
use hyper::server::{Request, Response};

pub struct CacheWrite;

impl CacheWrite {
    pub fn save(req: Request, res: Response) -> bool {
        // TODO: Not implemented

        if Self::is_cacheable(req, res) == true {
            // TODO: write cache (if memcached is down, ignore but throw a \
            //         log error)
            // TODO: implement support for Bloom-Response-TTL

            // Later:
            // TODO: implement support for Bloom-Response-Bucket
            // CONCERN: how to link this to the gen_ns() utility? We dont \
            //   know about which route is mapped to which bucket in advance. \
            //   so maybe redesign this part.

            true
        } else {
            // Not cacheable, ignore
            false
        }
    }

    fn is_cacheable(req: Request, res: Response) -> bool {
        Self::is_cacheable_method(req.method()) &&
            Self::is_cacheable_status(res.status()) &&
            Self::is_cacheable_response()
    }

    fn is_cacheable_method(method: &Method) -> bool {
        match *method {
            Method::Get | Method::Head => true,
            _ => false
        }
    }

    fn is_cacheable_status(status: StatusCode) -> bool {
        match status {
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

    fn is_cacheable_response() -> bool {
        // TODO: implement support for Bloom-Response-Ignore

        true
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
        assert_eq!(CacheWrite::is_cacheable_status(StatusCode::Ok),
            true, "200 OK");
        assert_eq!(CacheWrite::is_cacheable_status(StatusCode::Unauthorized),
            true, "401 OK");
        assert_eq!(CacheWrite::is_cacheable_status(StatusCode::BadRequest),
            false, "400 Bad Request");
        assert_eq!(CacheWrite::is_cacheable_status(StatusCode::InternalServerError),
            false, "500 Internal Server Error");
    }
}
