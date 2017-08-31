// Bloom
//
// HTTP REST API caching middleware
// Copyright: 2017, Valerian Saliou <valerian@valeriansaliou.name>
// License: Mozilla Public License v2.0 (MPL v2.0)

use futures::{future, Future};
use hyper::{Error, Client, Method, Uri, Headers, Body, Request};
use hyper::client::{HttpConnector, Response};

use server::listen::LISTEN_REMOTE;
use APP_CONF;

const MAX_SHARDS: u8 = 1;

lazy_static! {
    static ref SHARD_URI: Uri = format!("http://{}:{}", APP_CONF.proxy.inet.ip(),
        APP_CONF.proxy.inet.port()).parse().unwrap();
}

thread_local! {
    static TUNNEL_CLIENT: Client<HttpConnector> = Client::new(&LISTEN_REMOTE.lock().unwrap()
        .get_mut().to_owned().unwrap().handle().unwrap());
}

pub struct ProxyTunnelBuilder;

pub struct ProxyTunnel {
    shards: [Option<&'static Uri>; MAX_SHARDS as usize],
}

pub type ProxyTunnelFuture = Box<Future<Item = Response, Error = Error>>;

impl ProxyTunnelBuilder {
    pub fn new() -> ProxyTunnel {
        // We support only 1 shard for now.
        ProxyTunnel { shards: [Some(&*SHARD_URI)] }
    }
}

impl ProxyTunnel {
    pub fn run(
        &mut self,
        method: &Method,
        uri: &Uri,
        headers: &Headers,
        body: Body,
        shard: u8,
    ) -> ProxyTunnelFuture {
        if shard < MAX_SHARDS {
            // Route to target shard
            match self.shards[shard as usize] {
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
                            match method {
                                &Method::Post | &Method::Patch | &Method::Put => {
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
