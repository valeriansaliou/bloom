// Bloom
//
// HTTP REST API caching middleware
// Copyright: 2017, Valerian Saliou <valerian@valeriansaliou.name>
// License: Mozilla Public License v2.0 (MPL v2.0)

extern crate hyper;
extern crate futures;

use self::futures::future::FutureResult;

use self::hyper::header::{ContentLength, ContentType};
use self::hyper::server::{Service, Request, Response};

pub struct RequestHandle;

static MODULE: &'static str = "server:handle";

static DEFAULT_TEXT: &'static [u8] = b"{}";

impl Service for RequestHandle {
    type Request = Request;
    type Response = Response;
    type Error = hyper::Error;
    type Future = FutureResult<Response, hyper::Error>;

    fn call(&self, req: Request) -> Self::Future {
        info!("[{}] handled request: {} on {}", MODULE, req.method(),
            req.path());

        futures::future::ok(
            Response::new()
                .with_header(ContentLength(DEFAULT_TEXT.len() as u64))
                .with_header(ContentType::plaintext())
                .with_body(DEFAULT_TEXT)
        )
    }
}
