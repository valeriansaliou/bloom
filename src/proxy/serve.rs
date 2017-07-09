// Bloom
//
// HTTP REST API caching middleware
// Copyright: 2017, Valerian Saliou <valerian@valeriansaliou.name>
// License: Mozilla Public License v2.0 (MPL v2.0)

use futures::future;
use futures::future::FutureResult;
use hyper;
use hyper::{Method, StatusCode};
use hyper::server::{Request, Response};

use super::header::ProxyHeader;
use header::request_shard::HeaderRequestBloomRequestShard;
use header::status::{HeaderBloomStatus, HeaderBloomStatusValue};
use cache::read::CacheRead;
use cache::route::CacheRoute;

pub struct ProxyServeBuilder;

pub struct ProxyServe;

pub type ProxyServeFuture = FutureResult<Response, hyper::Error>;

impl ProxyServeBuilder {
    pub fn new() -> ProxyServe {
        ProxyServe {}
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

        future::ok(res)
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

        let ns = CacheRoute::gen_ns(shard, auth, req.version(), req.method(),
            req.path(), req.query());

        // TODO: support for 304 Not Modified here (return empty content \
        //   to ongoing specific client, but still read/populate cache normally)

        // TODO: implement support for Bloom-Response-Bucket
        // CONCERN: how to link this to the gen_ns() utility? We dont \
        //   know about which route is mapped to which bucket in advance. \
        //   so maybe redesign this part.

        info!("tunneling for ns = {}", ns);

        match CacheRead::acquire(ns.as_ref()) {
            Ok(cached_value) => {
                self.dispatch(res, HeaderBloomStatusValue::Hit, cached_value)
            },
            Err(_) => {
                // TODO -> connect to API using ConfigProxy[:shard].inet
                    // TODO -> enforce timeouts:
                        //   - ConfigProxy.connect_timeout
                        //   - ConfigProxy.read_timeout
                        //   - ConfigProxy.send_timeout
                // TODO -> CacheWrite::save(ns, req, res) (check return value)
                    // TODO -> return == true -> set 'Bloom-Status' as 'MISS'
                    // TODO -> return == false -> set 'Bloom-Status' as 'DIRECT'

                // TODO: not void
                let value_void = String::from("");

                // TODO
                self.dispatch(res, HeaderBloomStatusValue::Miss, value_void)
            }
        }

        debug!("done tunneling for ns = {}", ns);
    }

    fn dispatch(&self, res: &mut Response, bloomStatus: HeaderBloomStatusValue,
        value: String) {
        // TODO

        // TODO: tmp header
        // res.with_header(HeaderBloomStatus(bloomStatus))
        //     .with_body(value);
    }
}
