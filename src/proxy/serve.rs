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
use super::tunnel::ProxyTunnelBuilder;
use header::request_shard::HeaderRequestBloomRequestShard;
use header::status::HeaderBloomStatusValue;
use cache::read::CacheRead;
use cache::write::CacheWrite;
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

        let res = if req.headers().has::<HeaderRequestBloomRequestShard>() ==
            true {
            match *req.method() {
                Method::Options
                | Method::Head
                | Method::Get
                | Method::Post
                | Method::Patch
                | Method::Put
                | Method::Delete => {
                    self.accept(&req)
                }
                _ => {
                    self.reject(&req, StatusCode::MethodNotAllowed)
                }
            }
        } else {
            self.reject(&req, StatusCode::NotExtended)
        };

        future::ok(res)
    }

    fn accept(&self, req: &Request) -> Response {
        self.tunnel(req)
    }

    fn reject(&self, req: &Request, status: StatusCode) -> Response {
        Response::new()
            .with_status(status)
            .with_body(
                match *req.method() {
                    Method::Get
                    | Method::Post
                    | Method::Patch
                    | Method::Put => {
                        format!("{}", status)
                    }
                    _ => String::new()
                }
            )
    }

    fn tunnel(&self, req: &Request) -> Response {
        let (auth, shard) = ProxyHeader::parse_from_request(req.headers());

        let ns = CacheRoute::gen_ns(shard, auth, req.version(), req.method(),
            req.path(), req.query());

        // TODO: support for 304 Not Modified here (return empty content \
        //   to ongoing specific client, but still read/populate cache normally)

        // TODO: implement support for Bloom-Response-Bucket
        // CONCERN: how to link this to the gen_ns() utility? We dont \
        //   know about which route is mapped to which bucket in advance. \
        //   so maybe redesign this part.  <--- FOUND OUT
        // WAIT TO GO: any route can be 'tagged' as 'bucket' using a generic \
        //   tagging system. As buckets are only used for cache expiration, \
        //   and not cache storage, they are only useful as 'tags'. This way \
        //   we dont need to know them in advance.

        info!("tunneling for ns = {}", ns);

        match CacheRead::acquire(ns.as_ref()) {
            Ok(cached_value) => {
                self.dispatch_cached(cached_value)
            },
            Err(_) => {
                // TODO -> connect to API using ConfigProxy[:shard].inet
                    // TODO -> enforce timeouts:
                        //   - ConfigProxy.tunnel_connect_timeout
                        //   - ConfigProxy.tunnel_read_timeout
                        //   - ConfigProxy.tunnel_send_timeout
                // TODO -> CacheWrite::save(ns, req, res) (check return value)
                    // TODO -> return == true -> set 'Bloom-Status' as 'MISS'
                    // TODO -> return == false -> set 'Bloom-Status' as 'DIRECT'

                // Acquire response object from client to API

                // TODO: move this to a common factory, ie global.
                // CRITICAL: avoid spawning new threads and destroying them \
                //   for each connection.
                let mut tunnel = ProxyTunnelBuilder::new();

                match tunnel.run(req.method()) {
                    Ok(tunnel_res) => {
                        if CacheWrite::save(ns.as_ref(), req, &tunnel_res) ==
                            true {
                            self.dispatch_direct(tunnel_res,
                                HeaderBloomStatusValue::Miss)
                        } else {
                            self.dispatch_direct(tunnel_res,
                                HeaderBloomStatusValue::Direct)
                        }
                    }
                    _ => {
                        self.dispatch_failure()
                    }
                }
            }
        }
    }

    fn dispatch_cached(&self, value: String) -> Response {
        // TODO: handle 'tunnel_res: &mut Response' here.

        // TODO: issue w/ borrow
        // res.with_header(HeaderBloomStatus(HeaderBloomStatusValue::Hit));

        // TODO: parse value and split headers (restore them + set body)

        // TODO: append status
        // TODO: append headers

        Response::new()
            .with_status(StatusCode::Accepted)  // <-- TODO: dynamic status
            .with_body(value)
    }

    fn dispatch_direct(&self, tunnel_res: Response,
        bloomStatus: HeaderBloomStatusValue) -> Response {
        // TODO: handle 'tunnel_res: &mut Response' here.

        // res.set_header(HeaderBloomStatus(bloomStatus))

        // TODO: append status
        // TODO: append headers

        Response::new()
            .with_status(tunnel_res.status())  // <-- TODO: dynamic status
            .with_body(tunnel_res.body())
    }

    fn dispatch_failure(&self) -> Response {
        let status = StatusCode::BadGateway;

        // TODO: issue w/ borrow
        // res.with_header(HeaderBloomStatus(HeaderBloomStatusValue::Offline));

        Response::new()
            .with_status(status)
            .with_body(format!("{}", status))
    }
}
