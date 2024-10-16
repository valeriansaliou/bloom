// Bloom
//
// HTTP REST API caching middleware
// Copyright: 2017, Valerian Saliou <valerian@valeriansaliou.name>
// License: Mozilla Public License v2.0 (MPL v2.0)

use hyper::server::{Request, Response, Service};

use crate::proxy::serve::{ProxyServe, ProxyServeResponseFuture};

pub struct ServerRequestHandle;

impl Service for ServerRequestHandle {
    type Request = Request;
    type Response = Response;
    type Error = hyper::Error;
    type Future = ProxyServeResponseFuture;

    fn call(&self, req: Request) -> ProxyServeResponseFuture {
        debug!("called proxy serve");

        ProxyServe::handle(req)
    }
}
