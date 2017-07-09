// Bloom
//
// HTTP REST API caching middleware
// Copyright: 2017, Valerian Saliou <valerian@valeriansaliou.name>
// License: Mozilla Public License v2.0 (MPL v2.0)

use futures::future::FutureResult;
use hyper;
use hyper::server::{Service, Request, Response};

use proxy::serve::ProxyServeFuture;
use ::APP_PROXY_SERVE;

pub struct ServerRequestHandle;

impl ServerRequestHandle {
    pub fn new() -> ServerRequestHandle {
        ServerRequestHandle {}
    }
}

impl Service for ServerRequestHandle {
    type Request = Request;
    type Response = Response;
    type Error = hyper::Error;
    type Future = FutureResult<Response, hyper::Error>;

    fn call(&self, req: Request) -> ProxyServeFuture {
        APP_PROXY_SERVE.handle(req)
    }
}
