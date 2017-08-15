// Bloom
//
// HTTP REST API caching middleware
// Copyright: 2017, Valerian Saliou <valerian@valeriansaliou.name>
// License: Mozilla Public License v2.0 (MPL v2.0)

use hyper::{Error, Client, Method, Uri, Headers, Body, Request};
use hyper::client::HttpConnector;
use hyper::server::Response;
use tokio_core::reactor::Core;

use APP_CONF;

const MAX_SHARDS: u8 = 1;

lazy_static! {
    static ref SHARD_URI: Uri = format!("http://{}:{}", APP_CONF.proxy.inet.ip(),
        APP_CONF.proxy.inet.port()).parse().unwrap();
}

pub struct ProxyTunnelBuilder;

pub struct ProxyTunnel {
    core: Core,
    client: Client<HttpConnector>,
    shards: [Option<&'static Uri>; MAX_SHARDS as usize],
}

impl ProxyTunnelBuilder {
    pub fn new() -> ProxyTunnel {
        // TODO: keep a pool of connections active? (re-use existing connectors)
        let core = Core::new().unwrap();
        let handle = core.handle();
        let client = Client::configure()
            .connector(HttpConnector::new(APP_CONF.proxy.tunnel_threads, &handle))
            .build(&handle);

        // We support only 1 shard for now.
        ProxyTunnel {
            core: core,
            client: client,
            shards: [Some(&*SHARD_URI)],
        }
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
    ) -> Result<Response, Error> {
        if shard < MAX_SHARDS {
            // Route to target shard
            match self.shards[shard as usize] {
                Some(ref shard_uri) => {
                    match format!("{}{}", shard_uri, uri.path()).parse() {
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
                                    // TODO: blocking if non-empty, eg. if PATCH, why?
                                    tunnel_req.set_body(body);
                                }
                                _ => {}
                            }

                            self.core.run(self.client.request(tunnel_req))
                        }
                        Err(err) => Err(Error::Uri(err)),
                    }
                }
                None => Err(Error::Header),
            }
        } else {
            // Shard out of bounds
            Err(Error::Header)
        }
    }
}
