// Bloom
//
// HTTP REST API caching middleware
// Copyright: 2017, Valerian Saliou <valerian@valeriansaliou.name>
// License: Mozilla Public License v2.0 (MPL v2.0)

extern crate hyper;

use self::hyper::{Method, StatusCode};
use self::hyper::server::{Request, Response};

pub struct Write;

impl Write {
    pub fn save(ns: &str, req: Request, res: Response) {
        // TODO: Not implemented

        if Self::is_cacheable(req, res) == true {
            // TODO
        }
    }

    fn is_cacheable(req: Request, res: Response) -> bool {
        Self::is_cacheable_method(req) && Self::is_cacheable_status(res)
    }

    fn is_cacheable_method(req: Request) -> bool {
        match *req.method() {
            Method::Get | Method::Head => true,
            _ => false
        }
    }

    fn is_cacheable_status(res: Response) -> bool {
        match res.status() {
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
}

#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
        // TODO
    }
}
