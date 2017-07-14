// Bloom
//
// HTTP REST API caching middleware
// Copyright: 2017, Valerian Saliou <valerian@valeriansaliou.name>
// License: Mozilla Public License v2.0 (MPL v2.0)

use hyper::server::Http;

use super::handle::ServerRequestHandle;

use ::APP_CONF;

pub struct ServerListenBuilder;
pub struct ServerListen;

impl ServerListenBuilder {
    pub fn new() -> ServerListen {
        ServerListen {}
    }
}

impl ServerListen {
    pub fn run(&self) {
        let addr = APP_CONF.server.inet;
        let server = Http::new().bind(&addr, move || {
            debug!("handled new request");

            Ok(ServerRequestHandle)
        }).unwrap();

        info!("listening on http://{}", server.local_addr().unwrap());

        server.run().unwrap();
    }
}
