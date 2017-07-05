// Bloom
//
// HTTP REST API caching middleware
// Copyright: 2017, Valerian Saliou <valerian@valeriansaliou.name>
// License: Mozilla Public License v2.0 (MPL v2.0)

use std::sync::Arc;

use futures::future::FutureResult;
use hyper;
use hyper::server::{Service, Request, Response};

use cache::store::CacheStore;
use proxy::serve::ProxyServe;
use proxy::serve::ProxyServeFuture;

pub struct ServerRequestHandle {
    proxy_serve: Arc<ProxyServe>,
    cache_store: Arc<CacheStore>
}

impl ServerRequestHandle {
    pub fn new(proxy_serve: Arc<ProxyServe>, cache_store: Arc<CacheStore>) ->
        ServerRequestHandle {
        ServerRequestHandle {
            proxy_serve: proxy_serve,
            cache_store: cache_store
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
