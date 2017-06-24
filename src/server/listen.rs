// Bloom
//
// HTTP REST API caching middleware
// Copyright: 2017, Valerian Saliou <valerian@valeriansaliou.name>
// License: Mozilla Public License v2.0 (MPL v2.0)

extern crate hyper;

use self::hyper::server::Http;

use super::handle::RequestHandle;

use config::config::ConfigListen;

pub struct ListenBuilder;
pub struct Listen {
    config_listen: ConfigListen
}

impl ListenBuilder {
    pub fn new(config_listen: ConfigListen) -> Listen {
        Listen {
            config_listen: config_listen
        }
    }
}

impl Listen {
    pub fn run(&self) {
        let addr = self.config_listen.inet;
        let server = Http::new().bind(&addr, move || {
            Ok(RequestHandle)
        }).unwrap();

        info!("listening on http://{}", server.local_addr().unwrap());

        server.run().unwrap();
    }
}
