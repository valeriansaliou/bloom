// Bloom
//
// HTTP REST API caching middleware
// Copyright: 2017, Valerian Saliou <valerian@valeriansaliou.name>
// License: Mozilla Public License v2.0 (MPL v2.0)

use tokio_core::reactor::Core;
use futures;
use futures::future::FutureResult;
use hyper;
use hyper::Client;
use hyper::{Method, StatusCode};
use hyper::header::Basic;
use hyper::server::{Request, Response};

use super::header::ProxyHeader;
use config::config::ConfigProxy;
use cache::read::CacheRead;

pub struct ProxyServeBuilder;

#[derive(Clone)]
pub struct ProxyServe {
    config_proxy: ConfigProxy
}

pub type ProxyServeFuture = FutureResult<Response, hyper::Error>;

impl ProxyServeBuilder {
    pub fn new(config_proxy: ConfigProxy) -> ProxyServe {
        ProxyServe {
            config_proxy: config_proxy
        }
    }
}

impl ProxyServe {
    pub fn handle(&self, req: Request) -> ProxyServeFuture {
        let method = req.method();
        let path = req.path();

        info!("handled request: {} on {}", method, path);

        let mut res = Response::new();

        match *method {
            Method::Options
            | Method::Head
            | Method::Get
            | Method::Post
            | Method::Patch
            | Method::Put
            | Method::Delete => {
                self.accept(&req, &mut res)
            }
            _ => {
                self.reject(&req, &mut res)
            }
        }

        futures::future::ok(res)
    }

    fn accept(&self, req: &Request, res: &mut Response) {
        self.tunnel(req, res);
    }

    fn reject(&self, req: &Request, res: &mut Response) {
        res.set_status(StatusCode::MethodNotAllowed);
    }

    fn tunnel(&self, req: &Request, res: &mut Response) {
        let (auth, shard) = ProxyHeader::parse_from_request(req.headers());

        let ns = CacheRead::gen_ns(shard, req.version(), req.method(),
            req.path(), req.query(), auth);

        // TODO: CacheRead::acquire()
        // TODO -> if acquired, serve cached response
            // TODO -> set 'Bloom-Status' as 'HIT'
        // TODO -> else (not acquired); proxy connection
            // TODO -> CacheWrite::save() (check return value)
                // TODO -> return == true -> set 'Bloom-Status' as 'MISS'
                // TODO -> return == false -> set 'Bloom-Status' as 'DIRECT'

        debug!("tunneling for ns = {}", ns);
    }
}
