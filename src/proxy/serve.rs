// Bloom
//
// HTTP REST API caching middleware
// Copyright: 2017, Valerian Saliou <valerian@valeriansaliou.name>
// License: Mozilla Public License v2.0 (MPL v2.0)

extern crate hyper;
extern crate futures;

use self::futures::future::FutureResult;
use self::hyper::{Method, StatusCode};
use self::hyper::server::{Request, Response};

use config::config::ConfigProxy;

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
                self.accept(&mut res)
            }
            _ => {
                self.reject(&mut res)
            }
        }

        futures::future::ok(res)
    }

    pub fn accept(&self, res: &mut Response) {
        res.set_status(StatusCode::ServiceUnavailable);
    }

    pub fn reject(&self, res: &mut Response) {
        res.set_status(StatusCode::MethodNotAllowed);
    }
}
