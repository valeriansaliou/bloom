// Bloom
//
// HTTP REST API caching middleware
// Copyright: 2017, Valerian Saliou <valerian@valeriansaliou.name>
// License: Mozilla Public License v2.0 (MPL v2.0)

extern crate hyper;

use self::hyper::server::Http;

use super::handle::RequestHandle;

use config::config::ConfigListen;

pub struct ServerListenBuilder;
pub struct ServerListen {
    config_listen: ConfigListen
}

static MODULE: &'static str = "server:listen";

impl ServerListenBuilder {
    pub fn new(config_listen: ConfigListen) -> ServerListen {
        ServerListen {
            config_listen: config_listen
        }
    }
}

impl ServerListen {
    pub fn run(&self) {
        let addr = self.config_listen.inet;
        let server = Http::new().bind(&addr, || Ok(RequestHandle)).unwrap();

        info!("[{}] listening on http://{}", MODULE, server.local_addr().unwrap());

        server.run().unwrap();
    }
}
