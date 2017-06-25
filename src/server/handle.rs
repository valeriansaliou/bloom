// Bloom
//
// HTTP REST API caching middleware
// Copyright: 2017, Valerian Saliou <valerian@valeriansaliou.name>
// License: Mozilla Public License v2.0 (MPL v2.0)

extern crate hyper;
extern crate futures;

use self::futures::future::FutureResult;

use self::hyper::server::{Service, Request, Response};

use cache::store::CacheStore;
use proxy::serve::ProxyServe;
use proxy::serve::ProxyServeFuture;

pub struct ServerRequestHandle {
    proxy_serve: ProxyServe
}

impl ServerRequestHandle {
    pub fn new(proxy_serve: ProxyServe, cache_store: CacheStore) ->
        ServerRequestHandle {
        ServerRequestHandle {
            proxy_serve: proxy_serve
        }
    }
}

impl Service for ServerRequestHandle {
    type Request = Request;
    type Response = Response;
    type Error = hyper::Error;
    type Future = FutureResult<Response, hyper::Error>;

    fn call(&self, req: Request) -> ProxyServeFuture {
        self.proxy_serve.handle(req)
    }
}
