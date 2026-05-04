// Bloom
//
// HTTP REST API caching middleware
// Copyright: 2017, Valerian Saliou <valerian@valeriansaliou.name>
// License: Mozilla Public License v2.0 (MPL v2.0)

use bytes::Bytes;
use http::header::{HeaderMap, HeaderName, HeaderValue, ORIGIN};
use http::{Method, Response, StatusCode, Version};
use http_body_util::{BodyExt, Full};
use httparse;
use hyper::body::Incoming;
use hyper::Request;
use itertools::{Itertools, Position};
use tokio::sync::oneshot;

use super::header::ProxyHeader;
use super::tunnel::ProxyTunnel;
use crate::cache::read::CacheRead;
use crate::cache::route::CacheRoute;
use crate::cache::write::CacheWrite;
use crate::header::janitor::HeaderJanitor;
use crate::header::status::{HeaderBloomStatus, HeaderBloomStatusValue, HEADER_NAME as STATUS_HEADER_NAME};
use crate::LINE_FEED;

pub struct ProxyServe;

const CACHED_PARSE_MAX_HEADERS: usize = 100;

type BoxBody = Full<Bytes>;

impl ProxyServe {
    pub async fn handle(req: Request<Incoming>) -> Response<BoxBody> {
        info!("handled request: {} on {}", req.method(), req.uri().path());

        match *req.method() {
            Method::OPTIONS
            | Method::HEAD
            | Method::GET
            | Method::POST
            | Method::PATCH
            | Method::PUT
            | Method::DELETE => Self::accept(req).await,
            _ => Self::reject(StatusCode::METHOD_NOT_ALLOWED),
        }
    }

    async fn accept(req: Request<Incoming>) -> Response<BoxBody> {
        Self::tunnel(req).await
    }

    fn reject(status: StatusCode) -> Response<BoxBody> {
        let mut response = Response::builder()
            .status(status)
            .body(Full::new(Bytes::from(format!("{}", status))))
            .unwrap();

        response.headers_mut().insert(
            HeaderName::from_static(STATUS_HEADER_NAME),
            HeaderBloomStatus(HeaderBloomStatusValue::Reject).to_header_value(),
        );

        response
    }

    async fn tunnel(req: Request<Incoming>) -> Response<BoxBody> {
        let method = req.method().clone();
        let uri = req.uri().clone();
        let version = req.version();
        let headers = req.headers().clone();

        let (auth, shard) = ProxyHeader::parse_from_request(&headers);
        let auth_hash = CacheRoute::hash(&auth);

        let origin = headers
            .get(ORIGIN)
            .and_then(|v| v.to_str().ok())
            .map(|s| s.to_string());

        let (ns, ns_mask) = CacheRoute::gen_key_cache(
            shard,
            &auth_hash,
            version,
            &method,
            uri.path(),
            uri.query(),
            origin.as_deref(),
        );

        info!("tunneling for ns = {}", ns);

        let body_bytes = req
            .into_body()
            .collect()
            .await
            .map(|b| b.to_bytes())
            .unwrap_or_else(|_| Bytes::new());

        match Self::fetch_cached_data(shard, &ns, &method, &headers).await {
            Ok((fingerprint, cached_body)) => {
                Self::dispatch_cached(
                    shard,
                    ns,
                    ns_mask,
                    auth_hash,
                    method,
                    uri,
                    version,
                    headers,
                    body_bytes,
                    fingerprint,
                    cached_body,
                )
                .await
            }
            Err(_) => {
                Self::tunnel_over_proxy(
                    shard, ns, ns_mask, auth_hash, method, uri, version, headers, body_bytes,
                )
                .await
            }
        }
    }

    async fn fetch_cached_data(
        shard: u8,
        ns: &str,
        method: &Method,
        headers: &HeaderMap,
    ) -> Result<(String, Option<String>), ()> {
        match CacheRead::acquire_meta(shard, ns, method).await {
            Ok(fingerprint) => {
                debug!(
                    "got fingerprint for cached data = {} on ns = {}",
                    &fingerprint, ns
                );

                let if_none_match = headers
                    .get(http::header::IF_NONE_MATCH)
                    .and_then(|v| v.to_str().ok());

                let isnt_modified = match if_none_match {
                    Some(etag_value) => {
                        if etag_value == "*" {
                            true
                        } else {
                            let clean_etag = etag_value.trim_matches('"');
                            clean_etag == fingerprint
                        }
                    }
                    None => false,
                };

                debug!(
                    "got not modified status for cached data = {} on ns = {}",
                    isnt_modified, ns
                );

                Self::fetch_cached_data_body(ns.to_string(), fingerprint, !isnt_modified).await
            }
            Err(_) => Err(()),
        }
    }

    async fn fetch_cached_data_body(
        ns: String,
        fingerprint: String,
        do_acquire_body: bool,
    ) -> Result<(String, Option<String>), ()> {
        if !do_acquire_body {
            return Ok((fingerprint, None));
        }

        match CacheRead::acquire_body(&ns).await {
            Ok(Some(body)) => Ok((fingerprint, Some(body))),
            Ok(None) => {
                error!("failed fetching cached data body");
                Err(())
            }
            Err(_) => {
                error!("failed fetching cached data body");
                Err(())
            }
        }
    }

    async fn tunnel_over_proxy(
        shard: u8,
        ns: String,
        ns_mask: String,
        auth_hash: String,
        method: Method,
        uri: http::Uri,
        version: Version,
        headers: HeaderMap,
        body: Bytes,
    ) -> Response<BoxBody> {
        let (cancel_tx, cancel_rx) = oneshot::channel::<()>();

        let method_clone = method.clone();

        let tunnel_future = async {
            let result =
                ProxyTunnel::run(&method, &uri, &headers, body, shard, cancel_rx).await;

            match result {
                Ok(tunnel_res) => {
                    let write_result = CacheWrite::save(
                        ns,
                        ns_mask,
                        auth_hash,
                        shard,
                        method,
                        version,
                        tunnel_res.status,
                        tunnel_res.headers,
                        tunnel_res.body,
                    )
                    .await;

                    match write_result.body {
                        Ok(body_string) => Self::dispatch_fetched(
                            &method_clone,
                            &write_result.status,
                            write_result.headers,
                            HeaderBloomStatusValue::Miss,
                            body_string,
                            write_result.fingerprint,
                        ),
                        Err(body_string_values) => match body_string_values {
                            Some(body_string) => {
                                let mut headers = write_result.headers;
                                HeaderJanitor::clean(&mut headers);

                                Self::dispatch_fetched(
                                    &method_clone,
                                    &write_result.status,
                                    headers,
                                    HeaderBloomStatusValue::Direct,
                                    body_string,
                                    write_result.fingerprint,
                                )
                            }
                            None => Self::dispatch_failure(),
                        },
                    }
                }
                Err(_) => Self::dispatch_failure(),
            }
        };

        let response = tunnel_future.await;

        drop(cancel_tx);

        response
    }

    async fn dispatch_cached(
        shard: u8,
        ns: String,
        ns_mask: String,
        auth_hash: String,
        method: Method,
        req_uri: http::Uri,
        req_version: Version,
        req_headers: HeaderMap,
        req_body: Bytes,
        res_fingerprint: String,
        res_string: Option<String>,
    ) -> Response<BoxBody> {
        if let Some(res_string_value) = res_string {
            let mut headers = [httparse::EMPTY_HEADER; CACHED_PARSE_MAX_HEADERS];
            let mut res = httparse::Response::new(&mut headers);

            let body = Self::parse_response_body(&res_string_value);

            match res.parse(res_string_value.as_bytes()) {
                Ok(_) => {
                    let code = res.code.unwrap_or(500u16);
                    let status =
                        StatusCode::from_u16(code).unwrap_or(StatusCode::INTERNAL_SERVER_ERROR);

                    let mut response_headers = HeaderMap::new();

                    for header in res.headers.iter() {
                        if let (Ok(name), Ok(value)) = (
                            HeaderName::from_bytes(header.name.as_bytes()),
                            HeaderValue::from_bytes(header.value),
                        ) {
                            response_headers.insert(name, value);
                        }
                    }

                    ProxyHeader::set_etag(&mut response_headers, &res_fingerprint);

                    response_headers.insert(
                        HeaderName::from_static(STATUS_HEADER_NAME),
                        HeaderBloomStatus(HeaderBloomStatusValue::Hit).to_header_value(),
                    );

                    Self::respond(&method, status, response_headers, body)
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
                    .await
                }
            }
        } else {
            let mut headers = HeaderMap::new();

            ProxyHeader::set_etag(&mut headers, &res_fingerprint);
            headers.insert(
                HeaderName::from_static(STATUS_HEADER_NAME),
                HeaderBloomStatus(HeaderBloomStatusValue::Hit).to_header_value(),
            );

            Self::respond(&method, StatusCode::NOT_MODIFIED, headers, String::new())
        }
    }

    fn dispatch_fetched(
        method: &Method,
        status: &StatusCode,
        mut headers: HeaderMap,
        bloom_status: HeaderBloomStatusValue,
        body_string: String,
        fingerprint: Option<String>,
    ) -> Response<BoxBody> {
        if let Some(fingerprint_value) = fingerprint {
            ProxyHeader::set_etag(&mut headers, &fingerprint_value);
        }

        headers.insert(
            HeaderName::from_static(STATUS_HEADER_NAME),
            HeaderBloomStatus(bloom_status).to_header_value(),
        );

        Self::respond(method, *status, headers, body_string)
    }

    fn dispatch_failure() -> Response<BoxBody> {
        let status = StatusCode::BAD_GATEWAY;

        let mut headers = HeaderMap::new();
        headers.insert(
            HeaderName::from_static(STATUS_HEADER_NAME),
            HeaderBloomStatus(HeaderBloomStatusValue::Offline).to_header_value(),
        );

        Response::builder()
            .status(status)
            .body(Full::new(Bytes::from(format!("{}", status))))
            .unwrap()
    }

    fn parse_response_body(res_string_value: &str) -> String {
        let (mut body, mut is_last_line_empty) = (String::new(), false);

        let lines = res_string_value.lines().with_position();

        for (position, line) in lines {
            if body.is_empty() == false || is_last_line_empty == true {
                body.push_str(line);

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
        headers: HeaderMap,
        body_string: String,
    ) -> Response<BoxBody> {
        let mut builder = Response::builder().status(status);

        for (name, value) in headers.iter() {
            builder = builder.header(name, value);
        }

        match method {
            &Method::GET | &Method::POST | &Method::PATCH | &Method::PUT | &Method::DELETE => {
                builder.body(Full::new(Bytes::from(body_string))).unwrap()
            }
            _ => builder.body(Full::new(Bytes::new())).unwrap(),
        }
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
