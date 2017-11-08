// Bloom
//
// HTTP REST API caching middleware
// Copyright: 2017, Valerian Saliou <valerian@valeriansaliou.name>
// License: Mozilla Public License v2.0 (MPL v2.0)

use futures::future::{self, Future};
use httparse;
use hyper::{Error, Method, StatusCode, Headers};
use hyper::header::{Origin, IfNoneMatch, ETag, EntityTag};
use hyper::server::{Request, Response};

use super::header::ProxyHeader;
use super::tunnel::ProxyTunnel;
use header::request_shard::HeaderRequestBloomRequestShard;
use header::status::{HeaderBloomStatus, HeaderBloomStatusValue};
use cache::read::CacheRead;
use cache::write::CacheWrite;
use cache::route::CacheRoute;
use LINE_FEED;

pub struct ProxyServe;

const CACHED_PARSE_MAX_HEADERS: usize = 100;

pub type ProxyServeFuture = Box<Future<Item = Response, Error = Error>>;

impl ProxyServe {
    pub fn handle(req: Request) -> ProxyServeFuture {
        info!("handled request: {} on {}", req.method(), req.path());

        if req.headers().has::<HeaderRequestBloomRequestShard>() == true {
            match *req.method() {
                Method::Options | Method::Head | Method::Get | Method::Post | Method::Patch |
                Method::Put | Method::Delete => Self::accept(req),
                _ => Self::reject(req, StatusCode::MethodNotAllowed),
            }
        } else {
            Self::reject(req, StatusCode::NotExtended)
        }
    }

    fn accept(req: Request) -> ProxyServeFuture {
        Self::tunnel(req)
    }

    fn reject(req: Request, status: StatusCode) -> ProxyServeFuture {
        let mut headers = Headers::new();

        headers.set::<HeaderBloomStatus>(HeaderBloomStatus(HeaderBloomStatusValue::Reject));

        Self::respond(&req.method(), status, headers, format!("{}", status))
    }

    fn tunnel(req: Request) -> ProxyServeFuture {
        let (method, uri, version, headers, body) = req.deconstruct();
        let (headers, auth, shard) = ProxyHeader::parse_from_request(headers);

        let auth_hash = CacheRoute::hash(&auth);

        let (ns, ns_mask) = CacheRoute::gen_key_cache(
            shard,
            &auth_hash,
            version,
            &method,
            uri.path(),
            uri.query(),
            headers.get::<Origin>(),
        );

        info!("tunneling for ns = {}", ns);

        Box::new(
            CacheRead::acquire(&ns, &method)
                .or_else(|_| Err(Error::Incomplete))
                .and_then(move |result| {
                    match result {
                        Ok(value) => Self::dispatch_cached(&method, &headers, &value.0, value.1),
                        Err(_) => {
                            // Clone method value for closures. Sadly, it looks like Rust borrow \
                            //   checker doesnt discriminate properly on this check.
                            let method_success = method.to_owned();
                            let method_failure = method.to_owned();

                            Box::new(
                                ProxyTunnel::run(&method, &uri, &headers, body, shard)
                                    .and_then(move |tunnel_res| {
                                        CacheWrite::save(
                                            ns,
                                            ns_mask,
                                            auth_hash,
                                            shard,
                                            method,
                                            version,
                                            tunnel_res.status(),
                                            tunnel_res.headers().to_owned(),
                                            tunnel_res.body(),
                                        )
                                    })
                                    .and_then(move |result| match result.body {
                                        Ok(body_string) => {
                                            Self::dispatch_fetched(
                                                &method_success,
                                                &result.status,
                                                result.headers,
                                                HeaderBloomStatusValue::Miss,
                                                body_string,
                                                result.fingerprint,
                                            )
                                        }
                                        Err(body_string_values) => {
                                            match body_string_values {
                                                Some(body_string) => {
                                                    Self::dispatch_fetched(
                                                        &method_success,
                                                        &result.status,
                                                        result.headers,
                                                        HeaderBloomStatusValue::Direct,
                                                        body_string,
                                                        result.fingerprint,
                                                    )
                                                }
                                                _ => Self::dispatch_failure(&method_success),
                                            }
                                        }
                                    })
                                    .or_else(move |_| Self::dispatch_failure(&method_failure)),
                            )
                        }
                    }
                }),
        )
    }

    fn dispatch_cached(
        method: &Method,
        headers: &Headers,
        res_string: &str,
        res_fingerprint: String,
    ) -> ProxyServeFuture {
        // Check if not modified?
        let isnt_modified = match headers.get::<IfNoneMatch>() {
            Some(req_if_none_match) => {
                match *req_if_none_match {
                    IfNoneMatch::Any => true,
                    IfNoneMatch::Items(ref req_etags) => {
                        if let Some(req_etag) = req_etags.first() {
                            req_etag.weak_eq(&EntityTag::new(false, res_fingerprint.to_owned()))
                        } else {
                            false
                        }
                    }
                }
            }
            _ => false,
        };

        // Response not modified for client?
        if isnt_modified == true {
            // Process non-modified + cached headers
            let mut headers = Headers::new();

            ProxyHeader::set_etag(&mut headers, Self::fingerprint_etag(res_fingerprint));
            headers.set::<HeaderBloomStatus>(HeaderBloomStatus(HeaderBloomStatusValue::Hit));

            // Serve non-modified response
            return Self::respond(method, StatusCode::NotModified, headers, String::from(""));
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

                ProxyHeader::set_etag(&mut headers, Self::fingerprint_etag(res_fingerprint));
                headers.set::<HeaderBloomStatus>(HeaderBloomStatus(HeaderBloomStatusValue::Hit));

                // Serve cached response
                Self::respond(method, status, headers, res_body_string)
            }
            Err(err) => {
                error!("failed parsing cached response: {}", err);

                Self::dispatch_failure(method)
            }
        }
    }

    fn dispatch_fetched(
        method: &Method,
        status: &StatusCode,
        mut headers: Headers,
        bloom_status: HeaderBloomStatusValue,
        body_string: String,
        fingerprint: Option<String>,
    ) -> ProxyServeFuture {
        // Process ETag for content?
        if let Some(fingerprint_value) = fingerprint {
            ProxyHeader::set_etag(&mut headers, Self::fingerprint_etag(fingerprint_value));
        }

        headers.set(HeaderBloomStatus(bloom_status));

        Self::respond(method, *status, headers, body_string)
    }

    fn dispatch_failure(method: &Method) -> ProxyServeFuture {
        let status = StatusCode::BadGateway;

        let mut headers = Headers::new();

        headers.set::<HeaderBloomStatus>(HeaderBloomStatus(HeaderBloomStatusValue::Offline));

        Self::respond(method, status, headers, format!("{}", status))
    }

    fn fingerprint_etag(fingerprint: String) -> ETag {
        ETag(EntityTag::new(false, fingerprint))
    }

    fn respond(
        method: &Method,
        status: StatusCode,
        headers: Headers,
        body_string: String,
    ) -> ProxyServeFuture {
        Box::new(future::ok(match method {
            &Method::Get | &Method::Post | &Method::Patch | &Method::Put | &Method::Delete => {
                Response::new()
                    .with_status(status)
                    .with_headers(headers)
                    .with_body(body_string)
            }
            _ => Response::new().with_status(status).with_headers(headers),
        }))
    }
}
