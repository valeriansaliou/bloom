// Bloom
//
// HTTP REST API caching middleware
// Copyright: 2017, Valerian Saliou <valerian@valeriansaliou.name>
// License: Mozilla Public License v2.0 (MPL v2.0)

use futures;
use futures::future::FutureResult;
use hyper;
use hyper::{Method, StatusCode};
use hyper::server::{Request, Response};

use super::header::ProxyHeader;
use header::request::HeaderRequestBloomRequestShard;
use config::config::ConfigProxy;
use cache::route::CacheRoute;

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
        info!("handled request: {} on {}", req.method(), req.path());

        let mut res = Response::new();

        if req.headers().has::<HeaderRequestBloomRequestShard>() == true {
            match *req.method() {
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
                    self.reject(&req, &mut res, StatusCode::MethodNotAllowed)
                }
            }
        } else {
            self.reject(&req, &mut res, StatusCode::NotExtended)
        }

        futures::future::ok(res)
    }

    fn accept(&self, req: &Request, res: &mut Response) {
        self.tunnel(req, res);
    }

    fn reject(&self, req: &Request, res: &mut Response, status: StatusCode) {
        res.set_status(status);

        match *req.method() {
            Method::Get | Method::Post | Method::Patch | Method::Put => {
                res.set_body(format!("{}", status));
            }
            _ => {}
        }
    }

    fn tunnel(&self, req: &Request, res: &mut Response) {
        let (auth, shard) = ProxyHeader::parse_from_request(req.headers());

        let ns = CacheRoute::gen_ns(shard, req.version(), req.method(),
            req.path(), req.query(), auth);

        // TODO: support for 304 Not Modified here (return empty content \
        //   to ongoing specific client, but still read/populate cache normally)

        // TODO: CacheRead::acquire()
        // TODO -> if acquired, serve cached response
            // TODO -> set 'Bloom-Status' as 'HIT'
        // TODO -> else (not acquired); proxy connection
            // TODO -> connect to API using ConfigProxy[:shard].inet
                // TODO -> enforce timeouts:
                    //   - ConfigProxy.connect_timeout
                    //   - ConfigProxy.read_timeout
                    //   - ConfigProxy.send_timeout
            // TODO -> CacheWrite::save() (check return value)
                // TODO -> return == true -> set 'Bloom-Status' as 'MISS'
                // TODO -> return == false -> set 'Bloom-Status' as 'DIRECT'

        debug!("tunneling for ns = {}", ns);
    }
}
