// Bloom
//
// HTTP REST API caching middleware
// Copyright: 2017, Valerian Saliou <valerian@valeriansaliou.name>
// License: Mozilla Public License v2.0 (MPL v2.0)

extern crate hyper;
extern crate futures;

use self::futures::future::FutureResult;
use self::hyper::server::{Request, Response};

use config::config::ConfigProxy;

pub struct ServeBuilder;
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
        futures::future::ok(
            Response::new()
        )
    }
}
