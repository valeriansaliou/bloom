// Bloom
//
// HTTP REST API caching middleware
// Copyright: 2017, Valerian Saliou <valerian@valeriansaliou.name>
// License: Mozilla Public License v2.0 (MPL v2.0)

use hyper::server::Http;

use super::handle::ServerRequestHandle;

use config::config::ConfigListen;
use cache::store::CacheStore;
use proxy::serve::ProxyServe;

pub struct ServerListenBuilder;
pub struct ServerListen {
    config_listen: ConfigListen
}

impl ServerListenBuilder {
    pub fn new(config_listen: ConfigListen) -> ServerListen {
        ServerListen {
            config_listen: config_listen
        }
    }
}

impl ServerListen {
    pub fn run(&self, proxy_serve: ProxyServe, cache_store: CacheStore) {
        let addr = self.config_listen.inet;
        let server = Http::new().bind(&addr, move || {
            // TODO: solve those dirty clones?
            // CRITICAL, as this closure is called for EVERY HTTP request
            Ok(ServerRequestHandle::new(
                proxy_serve.clone(), cache_store.clone()
            ))
        }).unwrap();

        info!("listening on http://{}", server.local_addr().unwrap());

        server.run().unwrap();
    }
}
