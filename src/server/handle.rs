// Bloom
//
// HTTP REST API caching middleware
// Copyright: 2017, Valerian Saliou <valerian@valeriansaliou.name>
// License: Mozilla Public License v2.0 (MPL v2.0)

use hyper;
use hyper::server::{Service, Request, Response};

use proxy::serve::{ProxyServe, ProxyServeFuture};

pub struct ServerRequestHandle;

impl Service for ServerRequestHandle {
    type Request = Request;
    type Response = Response;
    type Error = hyper::Error;
    type Future = ProxyServeFuture;

    fn call(&self, req: Request) -> ProxyServeFuture {
        debug!("called proxy serve");

        ProxyServe::handle(req)
    }
}
