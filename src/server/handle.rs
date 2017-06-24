// Bloom
//
// HTTP REST API caching middleware
// Copyright: 2017, Valerian Saliou <valerian@valeriansaliou.name>
// License: Mozilla Public License v2.0 (MPL v2.0)

extern crate hyper;
extern crate futures;

use self::futures::future::FutureResult;

use self::hyper::server::{Service, Request, Response};

use proxy::serve::ServeFuture;
use proxy::serve::Serve;

pub struct RequestHandle;

impl Service for RequestHandle {
    type Request = Request;
    type Response = Response;
    type Error = hyper::Error;
    type Future = FutureResult<Response, hyper::Error>;

    fn call(&self, req: Request) -> ServeFuture {
        let method = req.method();
        let path = req.path();

        info!("handled request: {} on {}", method, path);

        futures::future::ok(
            Response::new()
        )
    }
}
