// Bloom
//
// HTTP REST API caching middleware
// Copyright: 2017, Valerian Saliou <valerian@valeriansaliou.name>
// License: Mozilla Public License v2.0 (MPL v2.0)

use std::sync::Arc;

use hyper::server::Http;

use super::handle::ServerRequestHandle;

use config::config::ConfigServer;
use cache::store::CacheStore;
use proxy::serve::ProxyServe;

pub struct ServerListenBuilder;
pub struct ServerListen {
    config_server: ConfigServer
}

impl ServerListenBuilder {
    pub fn new(config_server: ConfigServer) -> ServerListen {
        ServerListen {
            config_server: config_server
        }
    }
}

impl ServerListen {
    pub fn run(&self, proxy_serve: Arc<ProxyServe>,
        cache_store: Arc<CacheStore>) {
        let addr = self.config_server.inet;
        let server = Http::new().bind(&addr, move || {
            // TODO: can we make this better w/o cloning memory?
            // CRITICAL, as this closure is called for EVERY HTTP request
            Ok(ServerRequestHandle::new(
                proxy_serve.clone(), cache_store.clone()
            ))
        }).unwrap();

        info!("listening on http://{}", server.local_addr().unwrap());

        server.run().unwrap();
    }
}
