// Bloom
//
// HTTP REST API caching middleware
// Copyright: 2017, Valerian Saliou <valerian@valeriansaliou.name>
// License: Mozilla Public License v2.0 (MPL v2.0)

use hyper::{Error, Client, Uri, Method, Request};
use hyper::client::HttpConnector;
use hyper::server::Response;
use tokio_core::reactor::Core;

use ::APP_CONF;

pub struct ProxyTunnelBuilder;

pub struct ProxyTunnel {
    core: Core,
    client: Client<HttpConnector>,
    shards: [Uri; 1]
}

impl ProxyTunnelBuilder {
    pub fn new() -> ProxyTunnel {
        let mut core = Core::new().unwrap();
        let handle = core.handle();
        let client = Client::configure()
            .connector(HttpConnector::new(APP_CONF.proxy.tunnel_threads,
                &handle))
            .build(&handle);

        let shard_uri = format!("http://{}:{}", APP_CONF.proxy.inet.ip(),
                            APP_CONF.proxy.inet.port()).parse().unwrap();

        ProxyTunnel {
            core: core,
            client: client,
            shards: [shard_uri]
        }
    }
}

impl ProxyTunnel {
    pub fn run(&mut self, method: &Method) -> Result<Response, Error> {
        // TODO: multiple shard support (get from config.)

        let shard = 0;
        let tunnel_req = self.client.request(
            Request::new(method.clone(), self.shards[shard].clone()));

        self.core.run(tunnel_req)
    }
}
