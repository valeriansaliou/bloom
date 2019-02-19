// Bloom
//
// HTTP REST API caching middleware
// Copyright: 2017, Valerian Saliou <valerian@valeriansaliou.name>
// License: Mozilla Public License v2.0 (MPL v2.0)

use std::time::Duration;
use futures::{future, Future};
use hyper::{Error, Client, Method, Uri, Headers, Body, Request};
use hyper::client::{HttpConnector, Response};

use crate::server::listen::LISTEN_REMOTE;
use crate::APP_CONF;

const MAX_SHARDS: u8 = 16;
const CLIENT_KEEP_ALIVE_TIMEOUT_SECONDS: u64 = 30;

lazy_static! {
    static ref SHARD_REGISTER: [Option<Uri>; MAX_SHARDS as usize] = map_shards();
}

thread_local! {
    static TUNNEL_CLIENT: Client<HttpConnector> = make_client();
}

pub struct ProxyTunnel;

pub type ProxyTunnelFuture = Box<Future<Item = Response, Error = Error>>;

fn make_client() -> Client<HttpConnector> {
    Client::configure()
        .keep_alive(true)
        .keep_alive_timeout(Some(Duration::from_secs(CLIENT_KEEP_ALIVE_TIMEOUT_SECONDS)))
        .build(&LISTEN_REMOTE
            .lock()
            .unwrap()
            .get_mut()
            .to_owned()
            .unwrap()
            .handle()
            .unwrap())
}

fn map_shards() -> [Option<Uri>; MAX_SHARDS as usize] {
    // Notice: this array cannot be initialized using the short format, as hyper::Uri doesnt \
    //   implement the Copy trait, hence the ugly hardcoded initialization vector w/ Nones.
    let mut shards = [
        None,
        None,
        None,
        None,
        None,
        None,
        None,
        None,
        None,
        None,
        None,
        None,
        None,
        None,
        None,
        None,
    ];

    for shard in &APP_CONF.proxy.shard {
        // Shard number overflows?
        if shard.shard >= MAX_SHARDS {
            panic!("shard number overflows maximum of {} shards", MAX_SHARDS);
        }

        // Store this shard
        shards[shard.shard as usize] = Some(
            format!(
                "http://{}:{}",
                APP_CONF.proxy.shard[0].host,
                APP_CONF.proxy.shard[0].port
            ).parse()
                .expect("could not build shard uri"),
        );
    }

    shards
}

impl ProxyTunnel {
    pub fn run(
        method: &Method,
        uri: &Uri,
        headers: &Headers,
        body: Body,
        shard: u8,
    ) -> ProxyTunnelFuture {
        if shard < MAX_SHARDS {
            // Route to target shard
            match SHARD_REGISTER[shard as usize] {
                Some(ref shard_uri) => {
                    let mut tunnel_uri = format!("{}{}", shard_uri, uri.path());

                    if let Some(query) = uri.query() {
                        tunnel_uri.push_str("?");
                        tunnel_uri.push_str(query);
                    }

                    match tunnel_uri.parse() {
                        Ok(tunnel_uri) => {
                            let mut tunnel_req = Request::new(method.to_owned(), tunnel_uri);

                            // Forward headers
                            {
                                let tunnel_headers = tunnel_req.headers_mut();

                                tunnel_headers.clone_from(headers);
                            }

                            // Forward body?
                            // Notice: HTTP DELETE is not forbidden per-spec to hold a request \
                            //   body, even if it is not commonly used. Hence why we forward it.
                            match method {
                                &Method::Post | &Method::Patch | &Method::Put | &Method::Delete => {
                                    tunnel_req.set_body(body);
                                }
                                _ => {}
                            }

                            TUNNEL_CLIENT.with(|client| Box::new(client.request(tunnel_req)))
                        }
                        Err(err) => Box::new(future::err(Error::Uri(err))),
                    }
                }
                None => Box::new(future::err(Error::Header)),
            }
        } else {
            // Shard out of bounds
            Box::new(future::err(Error::Header))
        }
    }
}
