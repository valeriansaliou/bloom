// Bloom
//
// HTTP REST API caching middleware
// Copyright: 2017, Valerian Saliou <valerian@valeriansaliou.name>
// License: Mozilla Public License v2.0 (MPL v2.0)

extern crate hyper;
extern crate tokio_core;
extern crate futures;

use self::futures::future::FutureResult;
use self::tokio_core::reactor::Core;
use self::hyper::Client;
use self::hyper::{Method, StatusCode};
use self::hyper::server::{Request, Response};

use config::config::ConfigProxy;
use cache::read::Read;

pub struct ServeBuilder;

#[derive(Clone)]
pub struct Serve {
    config_proxy: ConfigProxy
}

pub type ServeFuture = FutureResult<Response, hyper::Error>;

impl ServeBuilder {
    pub fn new(config_proxy: ConfigProxy) -> Serve {
        Serve {
            config_proxy: config_proxy
        }
    }
}

impl Serve {
    pub fn handle(&self, req: Request) -> ServeFuture {
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
        // TODO: authorization
        // TODO: shard
        let ns = Read::gen_ns(0, req.method(), req.path(), "anon");

        debug!("tunneling for ns = {}", ns);
    }
}
