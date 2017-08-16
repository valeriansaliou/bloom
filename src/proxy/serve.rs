// Bloom
//
// HTTP REST API caching middleware
// Copyright: 2017, Valerian Saliou <valerian@valeriansaliou.name>
// License: Mozilla Public License v2.0 (MPL v2.0)

use futures::future::{self, Future};
use httparse;
use hyper;
use hyper::{Method, StatusCode, Headers};
use hyper::header::{Origin, IfNoneMatch, ETag, EntityTag};
use hyper::server::{Request, Response};
use farmhash;

use super::header::ProxyHeader;
use super::tunnel::ProxyTunnelBuilder;
use header::janitor::HeaderJanitor;
use header::request_shard::HeaderRequestBloomRequestShard;
use header::status::{HeaderBloomStatus, HeaderBloomStatusValue};
use cache::read::CacheRead;
use cache::write::CacheWrite;
use cache::route::CacheRoute;
use LINE_FEED;

pub struct ProxyServeBuilder;
pub struct ProxyServe;

const CACHED_PARSE_MAX_HEADERS: usize = 100;

pub type ProxyServeFuture = Box<Future<Item = Response, Error = hyper::Error>>;

impl ProxyServeBuilder {
    pub fn new() -> ProxyServe {
        ProxyServe {}
    }
}

impl ProxyServe {
    pub fn handle(&self, req: Request) -> ProxyServeFuture {
        info!("handled request: {} on {}", req.method(), req.path());

        if req.headers().has::<HeaderRequestBloomRequestShard>() == true {
            match *req.method() {
                Method::Options | Method::Head | Method::Get | Method::Post | Method::Patch |
                Method::Put | Method::Delete => self.accept(req),
                _ => self.reject(req, StatusCode::MethodNotAllowed),
            }
        } else {
            self.reject(req, StatusCode::NotExtended)
        }
    }

    fn accept(&self, req: Request) -> ProxyServeFuture {
        self.tunnel(req)
    }

    fn reject(&self, req: Request, status: StatusCode) -> ProxyServeFuture {
        let mut headers = Headers::new();

        headers.set::<HeaderBloomStatus>(HeaderBloomStatus(HeaderBloomStatusValue::Reject));

        self.respond(&req.method(), status, headers, format!("{}", status))
    }

    fn tunnel(&self, req: Request) -> ProxyServeFuture {
        let (method, uri, version, headers, body) = req.deconstruct();
        let (auth, shard) = ProxyHeader::parse_from_request(&headers);

        let ns = CacheRoute::gen_ns(
            shard,
            auth,
            version,
            &method,
            uri.path(),
            uri.query(),
            headers.get::<Origin>(),
        );

        info!("tunneling for ns = {}", ns);

        match CacheRead::acquire(ns.as_ref()) {
            Ok(cached_value) => self.dispatch_cached(&method, &headers, &cached_value),
            Err(_) => {
                match ProxyTunnelBuilder::new().run(&method, &uri, &headers, body, shard) {
                    Ok(tunnel_res) => {
                        let ref status = tunnel_res.status();
                        let headers = tunnel_res.headers().to_owned();

                        let result = CacheWrite::save(
                            ns.as_ref(),
                            &method,
                            &version,
                            status,
                            &headers,
                            tunnel_res.body(),
                        );

                        match result.body {
                            Ok(body_string) => {
                                self.dispatch_fetched(
                                    &method,
                                    status,
                                    headers,
                                    HeaderBloomStatusValue::Miss,
                                    body_string,
                                    result.value,
                                )
                            }
                            Err(body_string_values) => {
                                match body_string_values {
                                    Some(body_string) => {
                                        self.dispatch_fetched(
                                            &method,
                                            status,
                                            headers,
                                            HeaderBloomStatusValue::Direct,
                                            body_string,
                                            result.value,
                                        )
                                    }
                                    _ => self.dispatch_failure(&method),
                                }
                            }
                        }
                    }
                    _ => self.dispatch_failure(&method),
                }
            }
        }
    }

    fn dispatch_cached(
        &self,
        method: &Method,
        headers: &Headers,
        res_string: &str,
    ) -> ProxyServeFuture {
        // Process ETag for cached content
        let (res_hash, res_etag) = self.body_fingerprint(res_string);

        let isnt_modified = match headers.get::<IfNoneMatch>() {
            Some(req_if_none_match) => {
                (*req_if_none_match == IfNoneMatch::Items(vec![EntityTag::new(false, res_hash)]))
            }
            _ => false,
        };

        // Response not modified for client?
        if isnt_modified == true {
            // Process non-modified + cached headers
            let mut headers = Headers::new();

            headers.set::<ETag>(res_etag);
            headers.set::<HeaderBloomStatus>(HeaderBloomStatus(HeaderBloomStatusValue::Hit));

            // Serve non-modified response
            return self.respond(method, StatusCode::NotModified, headers, String::from(""));
        }

        // Response modified
        let mut headers = [httparse::EMPTY_HEADER; CACHED_PARSE_MAX_HEADERS];
        let mut res = httparse::Response::new(&mut headers);

        // Split headers from body
        let mut res_body_string = String::new();
        let mut is_last_line_empty = false;

        for res_line in res_string.lines() {
            if res_body_string.is_empty() == false || is_last_line_empty == true {
                // Write to body
                res_body_string.push_str(res_line.as_ref());
                res_body_string.push_str(LINE_FEED);
            }

            is_last_line_empty = res_line.is_empty();
        }

        match res.parse(res_string.as_bytes()) {
            Ok(_) => {
                // Process cached status
                let code = res.code.unwrap_or(500u16);
                let status = StatusCode::try_from(code).unwrap_or(StatusCode::Unregistered(code));

                // Process cached headers
                let mut headers = Headers::new();

                for header in res.headers {
                    if let (Ok(header_name), Ok(header_value)) =
                        (
                            String::from_utf8(Vec::from(header.name)),
                            String::from_utf8(Vec::from(header.value)),
                        )
                    {
                        headers.set_raw(header_name, header_value);
                    }
                }

                headers.set::<ETag>(res_etag);
                headers.set::<HeaderBloomStatus>(HeaderBloomStatus(HeaderBloomStatusValue::Hit));

                // Serve cached response
                self.respond(method, status, headers, res_body_string)
            }
            Err(err) => {
                error!("failed parsing cached response: {}", err);

                self.dispatch_failure(method)
            }
        }
    }

    fn dispatch_fetched(
        &self,
        method: &Method,
        status: &StatusCode,
        mut headers: Headers,
        bloom_status: HeaderBloomStatusValue,
        body_string: String,
        result_string: Option<String>,
    ) -> ProxyServeFuture {
        // Map headers to clean-up
        let mut headers_remove: Vec<String> = Vec::new();

        for header_view in headers.iter() {
            // Do not forward contextual and internal headers \
            //   (ie. 'Bloom-Response-*' headers)
            if HeaderJanitor::is_contextual(&header_view) == true ||
                HeaderJanitor::is_internal(&header_view) == true
            {
                headers_remove.push(String::from(header_view.name()));
            }
        }

        // Proceed headers clean-up
        for header_remove in headers_remove.iter() {
            headers.remove_raw(header_remove.as_ref());
        }

        // Process ETag for content?
        if let Some(result_string_value) = result_string {
            let (_, res_etag) = self.body_fingerprint(&result_string_value);

            headers.set::<ETag>(res_etag);
        }

        headers.set(HeaderBloomStatus(bloom_status));

        self.respond(method, *status, headers, body_string)
    }

    fn dispatch_failure(&self, method: &Method) -> ProxyServeFuture {
        let status = StatusCode::BadGateway;

        let mut headers = Headers::new();

        headers.set::<HeaderBloomStatus>(HeaderBloomStatus(HeaderBloomStatusValue::Offline));

        self.respond(method, status, headers, format!("{}", status))
    }

    fn body_fingerprint(&self, body_string: &str) -> (String, ETag) {
        let body_hash = format!("{:x}", farmhash::fingerprint64(body_string.as_bytes()));
        let body_etag = ETag(EntityTag::new(false, body_hash.to_owned()));

        (body_hash, body_etag)
    }

    fn respond(
        &self,
        method: &Method,
        status: StatusCode,
        headers: Headers,
        mut body_string: String,
    ) -> ProxyServeFuture {
        future::finished(
            match method {
                &Method::Get | &Method::Post | &Method::Patch | &Method::Put => {
                    // Ensure body string ends w/ a new line in any case, this \
                    //   fixes an 'infinite loop' issue w/ Hyper
                    if body_string.ends_with(LINE_FEED) == false {
                        body_string.push_str(LINE_FEED);
                    }

                    Response::new()
                        .with_status(status)
                        .with_headers(headers)
                        .with_body(body_string)
                }
                _ => Response::new().with_status(status).with_headers(headers),
            }
        ).boxed()
    }
}
