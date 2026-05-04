// Bloom
//
// HTTP REST API caching middleware
// Copyright: 2017, Valerian Saliou <valerian@valeriansaliou.name>
// License: Mozilla Public License v2.0 (MPL v2.0)

use bytes::Bytes;
use http::header::HeaderMap;
use http::{Method, StatusCode, Uri};
use http_body_util::{BodyExt, Full};
use hyper_util::client::legacy::connect::HttpConnector;
use hyper_util::client::legacy::Client;
use hyper_util::rt::TokioExecutor;
use std::sync::OnceLock;
use std::time::Duration;
use tokio::sync::oneshot;

use crate::APP_CONF;

const MAX_SHARDS: u8 = 16;
const CLIENT_POOL_IDLE_TIMEOUT_SECS: u64 = 30;

lazy_static! {
    static ref SHARD_REGISTER: [Option<Uri>; MAX_SHARDS as usize] = map_shards();
}

static TUNNEL_CLIENT: OnceLock<Client<HttpConnector, Full<Bytes>>> = OnceLock::new();

fn get_client() -> &'static Client<HttpConnector, Full<Bytes>> {
    TUNNEL_CLIENT.get_or_init(|| {
        let mut connector = HttpConnector::new();
        connector.set_keepalive(Some(Duration::from_secs(CLIENT_POOL_IDLE_TIMEOUT_SECS)));

        Client::builder(TokioExecutor::new())
            .pool_idle_timeout(Duration::from_secs(CLIENT_POOL_IDLE_TIMEOUT_SECS))
            .pool_max_idle_per_host(32)
            .build(connector)
    })
}

pub struct ProxyTunnel;

#[derive(Debug)]
pub enum TunnelError {
    ShardNotFound,
    InvalidUri,
    RequestFailed,
    BodyCollectFailed,
    ClientDisconnected,
}

pub struct TunnelResponse {
    pub status: StatusCode,
    pub headers: HeaderMap,
    pub body: Bytes,
}

fn map_shards() -> [Option<Uri>; MAX_SHARDS as usize] {
    let mut shards: [Option<Uri>; MAX_SHARDS as usize] = Default::default();

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
    pub async fn run(
        method: &Method,
        uri: &Uri,
        headers: &HeaderMap,
        body: Bytes,
        shard: u8,
        cancel_rx: oneshot::Receiver<()>,
    ) -> Result<TunnelResponse, TunnelError> {
        if shard >= MAX_SHARDS {
            return Err(TunnelError::ShardNotFound);
        }

        let shard_uri = match &SHARD_REGISTER[shard as usize] {
            Some(u) => u,
            None => return Err(TunnelError::ShardNotFound),
        };

        let mut tunnel_uri = format!(
            "{}://{}{}",
            shard_uri.scheme_str().unwrap_or("http"),
            shard_uri.authority().map(|a| a.as_str()).unwrap_or(""),
            uri.path()
        );

        if let Some(query) = uri.query() {
            tunnel_uri.push('?');
            tunnel_uri.push_str(query);
        }

        let tunnel_uri: Uri = tunnel_uri.parse().map_err(|_| TunnelError::InvalidUri)?;

        let mut builder = http::Request::builder()
            .method(method.clone())
            .uri(tunnel_uri);

        for (name, value) in headers.iter() {
            builder = builder.header(name, value);
        }

        // Forward body?
        // Notice: HTTP DELETE is not forbidden per-spec to hold a request body, even if it is
        //   not commonly used. Hence why we forward it.
        let req_body = match method {
            &Method::POST | &Method::PATCH | &Method::PUT | &Method::DELETE => Full::new(body),
            _ => Full::new(Bytes::new()),
        };

        let request = builder.body(req_body).map_err(|_| TunnelError::InvalidUri)?;

        let client = get_client();

        tokio::select! {
            result = client.request(request) => {
                match result {
                    Ok(response) => {
                        let status = response.status();
                        let headers = response.headers().clone();

                        let body = response
                            .into_body()
                            .collect()
                            .await
                            .map_err(|_| TunnelError::BodyCollectFailed)?
                            .to_bytes();

                        Ok(TunnelResponse {
                            status,
                            headers,
                            body,
                        })
                    }
                    Err(err) => {
                        error!("tunnel request failed: {:?}", err);
                        Err(TunnelError::RequestFailed)
                    }
                }
            }
            _ = cancel_rx => {
                debug!("tunnel cancelled by client disconnect, aborting upstream request");
                Err(TunnelError::ClientDisconnected)
            }
        }
    }
}
