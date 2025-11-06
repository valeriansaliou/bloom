// Bloom
//
// HTTP REST API caching middleware
// Copyright: 2017, Valerian Saliou <valerian@valeriansaliou.name>
// License: Mozilla Public License v2.0 (MPL v2.0)

use futures::future::{self, Future};
use httparse;
use hyper::header::{ETag, EntityTag, IfNoneMatch, Origin};
use hyper::server::{Request, Response};
use hyper::{Body, Error, Headers, HttpVersion, Method, StatusCode, Uri};
use itertools::{Itertools, Position};

use super::header::ProxyHeader;
use super::tunnel::ProxyTunnel;
use crate::cache::read::CacheRead;
use crate::cache::route::CacheRoute;
use crate::cache::write::CacheWrite;
use crate::header::janitor::HeaderJanitor;
use crate::header::status::{HeaderBloomStatus, HeaderBloomStatusValue};
use crate::LINE_FEED;

pub struct ProxyServe;

const CACHED_PARSE_MAX_HEADERS: usize = 100;

type ProxyServeResult = Result<(String, Option<String>), ()>;
type ProxyServeResultFuture = Box<dyn Future<Item = ProxyServeResult, Error = ()>>;

pub type ProxyServeResponseFuture = Box<dyn Future<Item = Response, Error = Error>>;

impl ProxyServe {
    pub fn handle(req: Request) -> ProxyServeResponseFuture {
        info!("handled request: {} on {}", req.method(), req.path());

        match *req.method() {
            Method::Options
            | Method::Head
            | Method::Get
            | Method::Post
            | Method::Patch
            | Method::Put
            | Method::Delete => Self::accept(req),
            _ => Self::reject(req, StatusCode::MethodNotAllowed),
        }
    }

    fn accept(req: Request) -> ProxyServeResponseFuture {
        Self::tunnel(req)
    }

    fn reject(req: Request, status: StatusCode) -> ProxyServeResponseFuture {
        let mut headers = Headers::new();

        headers.set::<HeaderBloomStatus>(HeaderBloomStatus(HeaderBloomStatusValue::Reject));

        Self::respond(&req.method(), status, headers, format!("{}", status))
    }

    fn tunnel(req: Request) -> ProxyServeResponseFuture {
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
            Self::fetch_cached_data(shard, &ns, &method, &headers)
                .or_else(|_| Err(Error::Incomplete))
                .and_then(move |result| match result {
                    Ok(value) => Self::dispatch_cached(
                        shard, ns, ns_mask, auth_hash, method, uri, version, headers, body,
                        value.0, value.1,
                    ),
                    Err(_) => Self::tunnel_over_proxy(
                        shard, ns, ns_mask, auth_hash, method, uri, version, headers, body,
                    ),
                }),
        )
    }

    fn fetch_cached_data(
        shard: u8,
        ns: &str,
        method: &Method,
        headers: &Headers,
    ) -> ProxyServeResultFuture {
        // Clone inner If-None-Match header value (pass it to future)
        let header_if_none_match = headers.get::<IfNoneMatch>().map(|value| value.to_owned());
        let ns_string = ns.to_string();

        Box::new(
            CacheRead::acquire_meta(shard, ns, method)
                .and_then(move |result| {
                    match result {
                        Ok(fingerprint) => {
                            debug!(
                                "got fingerprint for cached data = {} on ns = {}",
                                &fingerprint, &ns_string
                            );

                            // Check if not modified?
                            let isnt_modified = match header_if_none_match {
                                Some(ref req_if_none_match) => match req_if_none_match {
                                    &IfNoneMatch::Any => true,
                                    &IfNoneMatch::Items(ref req_etags) => {
                                        if let Some(req_etag) = req_etags.first() {
                                            req_etag.weak_eq(&EntityTag::new(
                                                false,
                                                fingerprint.to_owned(),
                                            ))
                                        } else {
                                            false
                                        }
                                    }
                                },
                                _ => false,
                            };

                            debug!(
                                "got not modified status for cached data = {} on ns = {}",
                                &isnt_modified, &ns_string
                            );

                            Self::fetch_cached_data_body(ns_string, fingerprint, !isnt_modified)
                        }
                        _ => Box::new(future::ok(Err(()))),
                    }
                })
                .or_else(|_| {
                    error!("failed fetching cached data meta");

                    future::ok(Err(()))
                }),
        )
    }

    fn fetch_cached_data_body(
        ns: String,
        fingerprint: String,
        do_acquire_body: bool,
    ) -> ProxyServeResultFuture {
        // Do not acquire body? (not modified)
        let body_fetcher = if do_acquire_body == false {
            Box::new(future::ok(Ok(None)))
        } else {
            // Will acquire body (modified)
            CacheRead::acquire_body(&ns)
        };

        Box::new(
            body_fetcher
                .and_then(|body_result| {
                    body_result
                        .or_else(|_| Err(()))
                        .map(|body| Ok((fingerprint, body)))
                })
                .or_else(|_| {
                    error!("failed fetching cached data body");

                    future::ok(Err(()))
                }),
        )
    }

    fn tunnel_over_proxy(
        shard: u8,
        ns: String,
        ns_mask: String,
        auth_hash: String,
        method: Method,
        uri: Uri,
        version: HttpVersion,
        headers: Headers,
        body: Body,
    ) -> ProxyServeResponseFuture {
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
                .and_then(move |mut result| match result.body {
                    Ok(body_string) => Self::dispatch_fetched(
                        &method_success,
                        &result.status,
                        result.headers,
                        HeaderBloomStatusValue::Miss,
                        body_string,
                        result.fingerprint,
                    ),
                    Err(body_string_values) => {
                        match body_string_values {
                            Some(body_string) => {
                                // Enforce clean headers, has usually they get \
                                //   cleaned from cache writer
                                HeaderJanitor::clean(&mut result.headers);

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

    fn dispatch_cached(
        shard: u8,
        ns: String,
        ns_mask: String,
        auth_hash: String,
        method: Method,
        req_uri: Uri,
        req_version: HttpVersion,
        req_headers: Headers,
        req_body: Body,
        res_fingerprint: String,
        res_string: Option<String>,
    ) -> ProxyServeResponseFuture {
        // Response modified? (non-empty body)
        if let Some(res_string_value) = res_string {
            let mut headers = [httparse::EMPTY_HEADER; CACHED_PARSE_MAX_HEADERS];
            let mut res = httparse::Response::new(&mut headers);

            // Split headers from body
            let body = Self::parse_response_body(&res_string_value);

            match res.parse(res_string_value.as_bytes()) {
                Ok(_) => {
                    // Process cached status
                    let code = res.code.unwrap_or(500u16);
                    let status =
                        StatusCode::try_from(code).unwrap_or(StatusCode::Unregistered(code));

                    // Process cached headers
                    let mut headers = Headers::new();

                    for header in res.headers {
                        if let (Ok(header_name), Ok(header_value)) = (
                            String::from_utf8(Vec::from(header.name)),
                            String::from_utf8(Vec::from(header.value)),
                        ) {
                            headers.set_raw(header_name, header_value);
                        }
                    }

                    ProxyHeader::set_etag(&mut headers, Self::fingerprint_etag(res_fingerprint));

                    headers
                        .set::<HeaderBloomStatus>(HeaderBloomStatus(HeaderBloomStatusValue::Hit));

                    // Serve cached response
                    Self::respond(&method, status, headers, body)
                }
                Err(err) => {
                    error!("failed parsing cached response: {}", err);

                    Self::tunnel_over_proxy(
                        shard,
                        ns,
                        ns_mask,
                        auth_hash,
                        method,
                        req_uri,
                        req_version,
                        req_headers,
                        req_body,
                    )
                }
            }
        } else {
            // Response not modified for client, process non-modified + cached headers
            let mut headers = Headers::new();

            ProxyHeader::set_etag(&mut headers, Self::fingerprint_etag(res_fingerprint));
            headers.set::<HeaderBloomStatus>(HeaderBloomStatus(HeaderBloomStatusValue::Hit));

            // Serve non-modified response
            Self::respond(&method, StatusCode::NotModified, headers, String::from(""))
        }
    }

    fn dispatch_fetched(
        method: &Method,
        status: &StatusCode,
        mut headers: Headers,
        bloom_status: HeaderBloomStatusValue,
        body_string: String,
        fingerprint: Option<String>,
    ) -> ProxyServeResponseFuture {
        // Process ETag for content?
        if let Some(fingerprint_value) = fingerprint {
            ProxyHeader::set_etag(&mut headers, Self::fingerprint_etag(fingerprint_value));
        }

        headers.set(HeaderBloomStatus(bloom_status));

        Self::respond(method, *status, headers, body_string)
    }

    fn dispatch_failure(method: &Method) -> ProxyServeResponseFuture {
        let status = StatusCode::BadGateway;

        let mut headers = Headers::new();

        headers.set::<HeaderBloomStatus>(HeaderBloomStatus(HeaderBloomStatusValue::Offline));

        Self::respond(method, status, headers, format!("{}", status))
    }

    fn fingerprint_etag(fingerprint: String) -> ETag {
        ETag(EntityTag::new(false, fingerprint))
    }

    fn parse_response_body(res_string_value: &str) -> String {
        let (mut body, mut is_last_line_empty) = (String::new(), false);

        // Scan response lines
        let lines = res_string_value.lines().with_position();

        for (position, line) in lines {
            if body.is_empty() == false || is_last_line_empty == true {
                // Append line to body
                body.push_str(line);

                // Append line feed character?
                if let Position::First | Position::Middle = position {
                    body.push_str(LINE_FEED);
                }
            }

            is_last_line_empty = line.is_empty();
        }

        body
    }

    fn respond(
        method: &Method,
        status: StatusCode,
        headers: Headers,
        body_string: String,
    ) -> ProxyServeResponseFuture {
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn it_parses_response_body() {
        let body = "2022-10-03";
        let headers =
            "Content-Type: text/plain; charset=utf-8\nServer: Kestrel\nTransfer-Encoding: chunked";

        let response_string = format!("{headers}\n\n{body}");

        assert_eq!(body, ProxyServe::parse_response_body(&response_string));
    }
}
