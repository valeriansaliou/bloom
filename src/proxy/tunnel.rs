// Bloom
//
// HTTP REST API caching middleware
// Copyright: 2017, Valerian Saliou <valerian@valeriansaliou.name>
// License: Mozilla Public License v2.0 (MPL v2.0)

use std::future::Future;
use std::pin::Pin;
use std::time::Duration;

use bytes::Bytes;
use http_body_util::{combinators::BoxBody, BodyExt, Empty};
use hyper::body::Incoming;
use hyper::header::HeaderMap;
use hyper::{Method, Request, Response, Uri};
use hyper_util::client::legacy::connect::HttpConnector;
use hyper_util::client::legacy::Client;
use hyper_util::rt::TokioExecutor;

use super::logger::ProxyLoggerRequest;
use super::serve::ProxyServeError;
use crate::{APP_CONF, APP_PROXY_LOGGER};

const MAX_SHARDS: u8 = 16;
const CLIENT_KEEP_ALIVE_TIMEOUT_SECONDS: u64 = 30;

lazy_static! {
    static ref SHARD_REGISTER: [Option<Uri>; MAX_SHARDS as usize] = map_shards();
}

thread_local! {
    static TUNNEL_CLIENT: Client<HttpConnector, ProxyTunnelRequestBody> = make_client();
}

pub struct ProxyTunnel;

type ProxyTunnelRequestBody = BoxBody<Bytes, ProxyServeError>;

type ProxyTunnelFuture =
    Pin<Box<dyn Future<Output = Result<Response<Incoming>, ProxyServeError>> + Send>>;

fn make_client() -> Client<HttpConnector, ProxyTunnelRequestBody> {
    Client::builder(TokioExecutor::new())
        .pool_idle_timeout(Duration::from_secs(CLIENT_KEEP_ALIVE_TIMEOUT_SECONDS))
        .build(HttpConnector::new())
}

fn map_shards() -> [Option<Uri>; MAX_SHARDS as usize] {
    // Notice: this array cannot be initialized using the short format, as hyper::Uri doesnt \
    //   implement the Copy trait, hence the ugly hardcoded initialization vector w/ Nones.
    let mut shards = [
        None, None, None, None, None, None, None, None, None, None, None, None, None, None, None,
        None,
    ];

    for shard in &APP_CONF.proxy.shard {
        // Shard number overflows?
        if shard.shard >= MAX_SHARDS {
            panic!("shard number overflows maximum of {} shards", MAX_SHARDS);
        }

        // Store this shard
        shards[shard.shard as usize] = Some(
            format!("http://{}:{}", shard.host, shard.port)
                .parse()
                .expect("could not build shard uri"),
        );
    }

    shards
}

impl ProxyTunnel {
    pub fn run(
        method: &Method,
        uri: &Uri,
        headers: &HeaderMap,
        body: Incoming,
        shard: u8,
    ) -> ProxyTunnelFuture {
        if shard < MAX_SHARDS {
            // Route to target shard
            match SHARD_REGISTER[shard as usize] {
                Some(ref shard_uri) => {
                    let mut tunnel_uri = format!(
                        "{}://{}{}",
                        shard_uri
                            .scheme()
                            .map(|scheme| scheme.as_str())
                            .unwrap_or("http"),
                        shard_uri
                            .authority()
                            .map(|authority| authority.as_str())
                            .unwrap_or(""),
                        uri.path()
                    );

                    if let Some(query) = uri.query() {
                        tunnel_uri.push_str("?");
                        tunnel_uri.push_str(query);
                    }

                    match tunnel_uri.parse::<Uri>() {
                        Ok(tunnel_uri) => {
                            // Forward body?
                            // Notice: HTTP DELETE is not forbidden per-spec to hold a request \
                            //   body, even if it is not commonly used. Hence why we forward it.
                            let req_body: ProxyTunnelRequestBody = match method {
                                &Method::POST | &Method::PATCH | &Method::PUT | &Method::DELETE => {
                                    body.map_err(|err| Box::new(err) as ProxyServeError).boxed()
                                }
                                _ => Empty::new().map_err(|_| unreachable!()).boxed(),
                            };

                            let mut tunnel_req = Request::new(req_body);

                            // Forward URI and method
                            *tunnel_req.method_mut() = method.clone();
                            *tunnel_req.uri_mut() = tunnel_uri;

                            // Forward headers
                            *tunnel_req.headers_mut() = headers.clone();

                            // Send request to request log? (if logger is enabled)
                            if let Some(ref proxy_logger) = *APP_PROXY_LOGGER {
                                proxy_logger
                                    .send(ProxyLoggerRequest {
                                        method: method.to_string(),
                                        uri: uri.to_string(),
                                        shard,
                                    })
                                    .ok();
                            }

                            TUNNEL_CLIENT.with(|client| {
                                let request = client.request(tunnel_req);

                                Box::pin(async move {
                                    request
                                        .await
                                        .map_err(|err| -> ProxyServeError { Box::new(err) })
                                }) as ProxyTunnelFuture
                            })
                        }
                        Err(_) => {
                            Box::pin(async move { Err(Self::make_proxy_err("invalid tunnel uri")) })
                        }
                    }
                }
                None => Box::pin(async move { Err(Self::make_proxy_err("shard not configured")) }),
            }
        } else {
            // Shard out of bounds
            Box::pin(async move { Err(Self::make_proxy_err("shard out of bounds")) })
        }
    }

    fn make_proxy_err(msg: &'static str) -> ProxyServeError {
        Box::new(std::io::Error::new(std::io::ErrorKind::Other, msg))
    }
}
