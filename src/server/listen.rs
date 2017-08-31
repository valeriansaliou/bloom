// Bloom
//
// HTTP REST API caching middleware
// Copyright: 2017, Valerian Saliou <valerian@valeriansaliou.name>
// License: Mozilla Public License v2.0 (MPL v2.0)

use std::cell::Cell;
use std::sync::{Arc, Mutex};
use hyper::server::Http;
use tokio_core::reactor::Remote;

use super::handle::ServerRequestHandle;
use APP_CONF;

lazy_static! {
    pub static ref LISTEN_REMOTE: Arc<Mutex<Cell<Option<Remote>>>> =
        Arc::new(Mutex::new(Cell::new(None)));
}

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
        let server = Http::new()
            .bind(&addr, move || {
                debug!("handled new request");

                Ok(ServerRequestHandle)
            })
            .unwrap();

        // Assign remote, used later on by the proxy client
        LISTEN_REMOTE
            .lock()
            .unwrap()
            .set(Some(server.handle().remote().to_owned()));

        info!("listening on http://{}", server.local_addr().unwrap());

        server.run().unwrap();
    }
}
