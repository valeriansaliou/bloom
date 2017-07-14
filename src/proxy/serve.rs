// Bloom
//
// HTTP REST API caching middleware
// Copyright: 2017, Valerian Saliou <valerian@valeriansaliou.name>
// License: Mozilla Public License v2.0 (MPL v2.0)

use futures::future;
use futures::future::FutureResult;
use httparse;
use hyper;
use hyper::{Method, StatusCode, Headers};
use hyper::server::{Request, Response};

use super::header::ProxyHeader;
use super::tunnel::ProxyTunnelBuilder;
use header::janitor::HeaderJanitor;
use header::request_shard::HeaderRequestBloomRequestShard;
use header::status::{HeaderBloomStatus, HeaderBloomStatusValue};
use cache::read::CacheRead;
use cache::write::CacheWrite;
use cache::route::CacheRoute;

pub struct ProxyServeBuilder;

pub struct ProxyServe;

const CACHED_PARSE_MAX_HEADERS: usize = 100;

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
        let mut headers = Headers::new();

        headers.set::<HeaderBloomStatus>(
            HeaderBloomStatus(HeaderBloomStatusValue::Reject));

        self.respond(req, status, headers, format!("{}", status))
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
        // WAY TO GO: any route can be 'tagged' as 'bucket' using a generic \
        //   tagging system. As buckets are only used for cache expiration, \
        //   and not cache storage, they are only useful as 'tags'. This way \
        //   we dont need to know them in advance.

        info!("tunneling for ns = {}", ns);

        match CacheRead::acquire(ns.as_ref()) {
            Ok(cached_value) => {
                self.dispatch_cached(req, cached_value)
            },
            Err(_) => {
                // TODO: move this to a common factory, ie global.
                // CRITICAL: avoid spawning new threads and destroying them \
                //   for each connection.
                let mut tunnel = ProxyTunnelBuilder::new();

                match tunnel.run(&req, shard) {
                    Ok(tunnel_res) => {
                        let ref status = tunnel_res.status();
                        let headers = tunnel_res.headers().clone();

                        match CacheWrite::save(ns.as_ref(),
                            req, status, &headers, tunnel_res.body()) {
                            Ok(body_string) => {
                                self.dispatch_fetched(req, status, headers,
                                    HeaderBloomStatusValue::Miss, body_string)
                            }
                            Err(body_string_values) => {
                                match body_string_values {
                                    Some(body_string) => {
                                        self.dispatch_fetched(req, status,
                                            headers,
                                            HeaderBloomStatusValue::Direct,
                                            body_string)
                                    }
                                    _ => self.dispatch_failure(req)
                                }
                            }
                        }
                    }
                    _ => {
                        self.dispatch_failure(req)
                    }
                }
            }
        }
    }

    fn dispatch_cached(&self, req: &Request, res_string: String) -> Response {
        let mut headers = [httparse::EMPTY_HEADER; CACHED_PARSE_MAX_HEADERS];
        let mut res = httparse::Response::new(&mut headers);

        // Split headers from body
        let mut res_body_string = String::new();
        let mut is_last_line_empty = false;

        for res_line in res_string.lines() {
            if res_body_string.is_empty() == false ||
                is_last_line_empty == true {
                // Write to body
                res_body_string.push_str(res_line.as_ref());
                res_body_string.push_str("\r\n");
            }

            is_last_line_empty = res_line.is_empty();
        }

        match res.parse(res_string.as_bytes()) {
            Ok(_) => {
                // Process cached status
                let code = res.code.unwrap_or(500u16);
                let status = StatusCode::try_from(code)
                                .unwrap_or(StatusCode::Unregistered(code));

                // Process cached headers
                let mut headers = Headers::new();

                for header in res.headers {
                    headers.set_raw(
                        String::from_utf8(Vec::from(header.name)).unwrap(),
                        String::from_utf8(Vec::from(header.value)).unwrap()
                    );
                }

                headers.set::<HeaderBloomStatus>(
                    HeaderBloomStatus(HeaderBloomStatusValue::Hit));

                // Serve cached response
                self.respond(req, status, headers, res_body_string)
            }
            Err(err) => {
                error!("failed parsing cached response: {}", err);

                self.dispatch_failure(req)
            }
        }
    }

    fn dispatch_fetched(&self, req: &Request, status: &StatusCode,
        mut headers: Headers, bloom_status: HeaderBloomStatusValue,
        body_string: String) ->
        Response {
        // Map headers to clean-up
        let mut headers_remove: Vec<String> = Vec::new();

        for header_view in headers.iter() {
            // Do not forward contextual and internal headers \
            //   (ie. 'Bloom-Response-*' headers)
            if HeaderJanitor::is_contextual(&header_view) == true ||
                HeaderJanitor::is_internal(&header_view) == true {
                headers_remove.push(String::from(header_view.name()));
            }
        }

        // Proceed headers clean-up
        for header_remove in headers_remove.iter() {
            headers.remove_raw(header_remove.as_ref());
        }

        headers.set(HeaderBloomStatus(bloom_status));

        self.respond(req, *status, headers, body_string)
    }

    fn dispatch_failure(&self, req: &Request) -> Response {
        let status = StatusCode::BadGateway;

        let mut headers = Headers::new();

        headers.set::<HeaderBloomStatus>(
            HeaderBloomStatus(HeaderBloomStatusValue::Offline));

        self.respond(req, status, headers, format!("{}", status))
    }

    fn respond(&self, req: &Request, status: StatusCode, headers: Headers,
        body_string: String) -> Response {
        match *req.method() {
            Method::Get
            | Method::Post
            | Method::Patch
            | Method::Put => {
                Response::new()
                    .with_status(status)
                    .with_headers(headers)
                    .with_body(body_string)
            }
            _ => {
                Response::new()
                    .with_status(status)
                    .with_headers(headers)
            }
        }
    }
}
