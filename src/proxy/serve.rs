// Bloom
//
// HTTP REST API caching middleware
// Copyright: 2017, Valerian Saliou <valerian@valeriansaliou.name>
// License: Mozilla Public License v2.0 (MPL v2.0)

use std::future::Future;
use std::pin::Pin;
use std::time::Instant;

use bytes::Bytes;
use http_body_util::Full;
use httparse;
use hyper::body::Incoming;
use hyper::header::{self, HeaderMap, HeaderName, HeaderValue};
use hyper::{Method, Request, Response, StatusCode, Uri, Version};
use itertools::{Itertools, Position};

use super::header::ProxyHeader;
use super::lock::ProxyLock;
use super::tunnel::ProxyTunnel;
use crate::cache::check::CacheCheck;
use crate::cache::read::CacheRead;
use crate::cache::route::CacheRoute;
use crate::cache::write::CacheWrite;
use crate::header::janitor::HeaderJanitor;
use crate::header::status::{HeaderBloomStatus, HeaderBloomStatusValue};
use crate::{APP_CONF, LINE_FEED};

pub struct ProxyServe;

const CACHED_PARSE_MAX_HEADERS: usize = 100;

pub type ProxyServeError = Box<dyn std::error::Error + Send + Sync + 'static>;

pub type ProxyServeResponseFuture =
    Pin<Box<dyn Future<Output = Result<Response<Full<Bytes>>, ProxyServeError>> + Send>>;

impl ProxyServe {
    pub fn handle(req: Request<Incoming>) -> ProxyServeResponseFuture {
        info!("handled request: {} on {}", req.method(), req.uri().path());

        match req.method() {
            &Method::OPTIONS
            | &Method::HEAD
            | &Method::GET
            | &Method::POST
            | &Method::PATCH
            | &Method::PUT
            | &Method::DELETE => Self::accept(req),
            _ => Self::reject(req, StatusCode::METHOD_NOT_ALLOWED),
        }
    }

    fn accept(req: Request<Incoming>) -> ProxyServeResponseFuture {
        Self::tunnel(req)
    }

    fn reject(req: Request<Incoming>, status: StatusCode) -> ProxyServeResponseFuture {
        let mut headers = HeaderMap::new();

        headers.insert(
            HeaderBloomStatus::header_name(),
            HeaderBloomStatus(HeaderBloomStatusValue::Reject).to_header_value(),
        );

        Box::pin(Self::respond(
            req.method().clone(),
            status,
            headers,
            format!("{}", status),
        ))
    }

    fn tunnel(req: Request<Incoming>) -> ProxyServeResponseFuture {
        let (parts, body) = req.into_parts();

        let method = parts.method;
        let uri = parts.uri;
        let version = parts.version;

        let (headers, auth, shard) = ProxyHeader::parse_from_request(parts.headers);

        let auth_hash = CacheRoute::hash(&auth);

        let origin = headers
            .get(header::ORIGIN)
            .and_then(|origin| origin.to_str().ok());

        let (ns, ns_mask) = CacheRoute::gen_key_cache(
            shard,
            &auth_hash,
            version,
            &method,
            uri.path(),
            uri.query(),
            origin,
        );

        info!("tunneling for ns = {}", ns);

        Box::pin(async move {
            let fetch_result = Self::fetch_cached_data(shard, &ns, &method, &headers)
                .await
                .map_err(|_| Self::make_proxy_error("fetch error"))?;

            match fetch_result {
                Ok(value) => {
                    Self::dispatch_cached(
                        shard, ns, ns_mask, auth_hash, method, uri, version, headers, body,
                        value.0, value.1,
                    )
                    .await
                }
                Err(_) => {
                    Self::queue_tunnel_over_proxy(
                        shard, ns, ns_mask, auth_hash, method, uri, version, headers, body,
                    )
                    .await
                }
            }
        })
    }

    async fn fetch_cached_data(
        shard: u8,
        ns: &str,
        method: &Method,
        headers: &HeaderMap,
    ) -> Result<Result<(String, Option<String>), ()>, ()> {
        let header_if_none_match = headers
            .get(header::IF_NONE_MATCH)
            .and_then(|value| value.to_str().ok())
            .map(|value| value.to_owned());

        let ns_string = ns.to_string();

        match CacheRead::acquire_meta(shard, ns, method).await {
            Ok(Ok((fingerprint, is_body_compressed))) => {
                debug!(
                    "got fingerprint for cached data = {} on ns = {}",
                    &fingerprint, &ns_string
                );

                // Check if not modified?
                let isnt_modified = match &header_if_none_match {
                    Some(if_none_match_value) => {
                        ProxyHeader::check_if_none_match(if_none_match_value, &fingerprint)
                    }
                    None => false,
                };

                debug!(
                    "got not modified status for cached data = {} on ns = {}",
                    &isnt_modified, &ns_string
                );

                Self::fetch_cached_data_body(
                    ns_string,
                    fingerprint,
                    !isnt_modified,
                    is_body_compressed,
                )
                .await
            }
            Ok(Err(_)) => Ok(Err(())),
            Err(_) => {
                error!("failed fetching cached data meta");

                Ok(Err(()))
            }
        }
    }

    async fn fetch_cached_data_body(
        ns: String,
        fingerprint: String,
        do_acquire_body: bool,
        is_body_compressed: bool,
    ) -> Result<Result<(String, Option<String>), ()>, ()> {
        // Do not acquire body? (not modified)
        if do_acquire_body == false {
            return Ok(Ok((fingerprint, None)));
        }

        // Will acquire body (modified)
        match CacheRead::acquire_body(&ns, is_body_compressed).await {
            Ok(Ok(body)) => Ok(Ok((fingerprint, body))),
            Ok(Err(_)) => {
                error!("failed fetching cached data body");

                Ok(Err(()))
            }
            Err(_) => {
                error!("failed fetching cached data body");

                Ok(Err(()))
            }
        }
    }

    async fn queue_tunnel_over_proxy(
        shard: u8,
        ns: String,
        ns_mask: String,
        auth_hash: String,
        method: Method,
        uri: Uri,
        version: Version,
        headers: HeaderMap,
        body: Incoming,
    ) -> Result<Response<Full<Bytes>>, ProxyServeError> {
        debug!("queue request for tunnelling with ns = {}", ns);

        // Acquire per-cache namespace lock so parallel requests on the same \
        //   path queue up; the lock is held until the first tunnelling \
        //   completes and hopefully leads to cached data being written,
        //   ensuring subsequent queued requests get a cache hit.
        // Important #1: only for cacheable requests! It is pointless to hoard \
        //   on locks for non-cacheable requests, those can safely be \
        //   performed in parallel since there is no benefit in locking here.
        // Important #2: the lock guard HAS TO BE RETURNED so that it is not \
        //   immediately dropped. We need the drop to occur when this method \
        //   returns. It is NOT used, ONLY returned for ownership reasons.
        let _lock_guard = if APP_CONF.proxy.lock_tunnel_path == true
            && CacheCheck::from_request(&method) == true
        {
            // Acquire slowlog vector (start time and slowlog threshold, if \
            //   enabled)
            let slowlog_vector = if let Some(slowlog_millis) = APP_CONF.proxy.lock_slowlog_millis {
                Some((Instant::now(), slowlog_millis))
            } else {
                None
            };

            // Acquire the proxy lock (and wait if locked)
            let (lock_guard, had_to_wait_for_lock) = ProxyLock::acquire(&ns).await;

            debug!(
                "request obtained queue lock for immediate tunnelling for ns = {} (queued: {})",
                ns, had_to_wait_for_lock
            );

            // Check if should log to slowlog?
            if let Some((wait_start, slowlog_millis)) = slowlog_vector {
                let waited_millis = wait_start.elapsed().as_millis() as u64;

                // Log request to slow log? (because it spent a long time in \
                //   queue, waiting for a previous request to complete)
                if waited_millis > slowlog_millis {
                    warn!(
                        "slow request — waited {}ms in queue for ns = ${}, path: {}",
                        waited_millis,
                        ns,
                        uri.path()
                    );
                }
            }

            // Double-check cache if we had to wait for lock; cause that means \
            //   a previous request may have already populated the Bloom cache \
            //   while we were queued. Skip if we were first (lock was free) \
            //   since the cache is guaranteed empty and we want to spare a \
            //   roundtrip to Redis.
            if had_to_wait_for_lock == true {
                let fetch_result_recheck = Self::fetch_cached_data(shard, &ns, &method, &headers)
                    .await
                    .map_err(|_| Self::make_proxy_error("fetch error (re-check)"))?;

                // Cache has been populated while waiting? Serve from cache!
                if let Ok(value) = fetch_result_recheck {
                    debug!(
                        "response for queued request is now in cache for ns = {}",
                        ns
                    );

                    return Self::dispatch_cached(
                        shard, ns, ns_mask, auth_hash, method, uri, version, headers, body,
                        value.0, value.1,
                    )
                    .await;
                } else {
                    debug!(
                        "response for queued request is still not in cache for ns = {}",
                        ns
                    );
                }
            }

            Some(lock_guard)
        } else {
            debug!(
                "request will be sent for immediate tunnelling for ns = {}",
                ns
            );

            None
        };

        // Dispatch request to the downstream API server
        Self::dispatch_tunnel(
            shard, ns, ns_mask, auth_hash, method, uri, version, headers, body,
        )
        .await
    }

    async fn dispatch_tunnel(
        shard: u8,
        ns: String,
        ns_mask: String,
        auth_hash: String,
        method: Method,
        uri: Uri,
        version: Version,
        headers: HeaderMap,
        body: Incoming,
    ) -> Result<Response<Full<Bytes>>, ProxyServeError> {
        // Clone method value for closures. Sadly, it looks like Rust borrow \
        //   checker doesnt discriminate properly on this check.
        let method_success = method.to_owned();
        let method_failure = method.to_owned();

        let tunnel_result: Result<_, ProxyServeError> =
            ProxyTunnel::run(&method, &uri, &headers, body, shard).await;

        match tunnel_result {
            Ok(tunnel_res) => {
                let write_result = CacheWrite::save(
                    ns,
                    ns_mask,
                    auth_hash,
                    shard,
                    method,
                    version,
                    tunnel_res.status(),
                    tunnel_res.headers().to_owned(),
                    tunnel_res.into_body(),
                )
                .await;

                match write_result {
                    Ok(mut result) => match result.body {
                        Ok(body_string) => {
                            Self::dispatch_fetched(
                                &method_success,
                                &result.status,
                                result.headers,
                                HeaderBloomStatusValue::Miss,
                                body_string,
                                result.fingerprint,
                            )
                            .await
                        }
                        Err(body_string_values) => match body_string_values {
                            Some(body_string) => {
                                // Enforce clean headers, as usually they get \
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
                                .await
                            }
                            _ => Self::dispatch_failure(&method_success).await,
                        },
                    },
                    Err(_) => Self::dispatch_failure(&method_failure).await,
                }
            }
            Err(_) => Self::dispatch_failure(&method_failure).await,
        }
    }

    async fn dispatch_cached(
        shard: u8,
        ns: String,
        ns_mask: String,
        auth_hash: String,
        method: Method,
        req_uri: Uri,
        req_version: Version,
        req_headers: HeaderMap,
        req_body: Incoming,
        res_fingerprint: String,
        res_string: Option<String>,
    ) -> Result<Response<Full<Bytes>>, ProxyServeError> {
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
                        StatusCode::from_u16(code).unwrap_or(StatusCode::INTERNAL_SERVER_ERROR);

                    // Process cached headers
                    let mut headers = HeaderMap::new();

                    for header in res.headers {
                        if let (Ok(header_name), Ok(header_value)) = (
                            HeaderName::from_bytes(header.name.as_bytes()),
                            HeaderValue::from_bytes(header.value),
                        ) {
                            headers.insert(header_name, header_value);
                        }
                    }

                    ProxyHeader::set_etag(&mut headers, &res_fingerprint);

                    headers.insert(
                        HeaderBloomStatus::header_name(),
                        HeaderBloomStatus(HeaderBloomStatusValue::Hit).to_header_value(),
                    );

                    // Serve cached response
                    Self::respond(method, status, headers, body).await
                }
                Err(err) => {
                    error!("failed parsing cached response: {}", err);

                    Self::dispatch_tunnel(
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
            // Response not modified for client, process non-modified + cached headers
            let mut headers = HeaderMap::new();

            ProxyHeader::set_etag(&mut headers, &res_fingerprint);

            headers.insert(
                HeaderBloomStatus::header_name(),
                HeaderBloomStatus(HeaderBloomStatusValue::Hit).to_header_value(),
            );

            // Serve non-modified response
            Self::respond(method, StatusCode::NOT_MODIFIED, headers, String::from("")).await
        }
    }

    async fn dispatch_fetched(
        method: &Method,
        status: &StatusCode,
        mut headers: HeaderMap,
        bloom_status: HeaderBloomStatusValue,
        body_string: String,
        fingerprint: Option<String>,
    ) -> Result<Response<Full<Bytes>>, ProxyServeError> {
        // Process ETag for content?
        if let Some(fingerprint_value) = fingerprint {
            ProxyHeader::set_etag(&mut headers, &fingerprint_value);
        }

        headers.insert(
            HeaderBloomStatus::header_name(),
            HeaderBloomStatus(bloom_status).to_header_value(),
        );

        Self::respond(method.clone(), *status, headers, body_string).await
    }

    async fn dispatch_failure(method: &Method) -> Result<Response<Full<Bytes>>, ProxyServeError> {
        let status = StatusCode::BAD_GATEWAY;

        let mut headers = HeaderMap::new();

        headers.insert(
            HeaderBloomStatus::header_name(),
            HeaderBloomStatus(HeaderBloomStatusValue::Offline).to_header_value(),
        );

        Self::respond(method.clone(), status, headers, format!("{}", status)).await
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

    fn make_proxy_error(msg: &'static str) -> ProxyServeError {
        Box::new(std::io::Error::new(std::io::ErrorKind::Other, msg))
    }

    async fn respond(
        method: Method,
        status: StatusCode,
        headers: HeaderMap,
        body_string: String,
    ) -> Result<Response<Full<Bytes>>, ProxyServeError> {
        let body = match method {
            Method::GET | Method::POST | Method::PATCH | Method::PUT | Method::DELETE => {
                Full::new(Bytes::from(body_string))
            }
            _ => Full::new(Bytes::new()),
        };

        let mut response = Response::new(body);

        *response.status_mut() = status;
        *response.headers_mut() = headers;

        Ok(response)
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
