// Bloom
//
// HTTP REST API caching middleware
// Copyright: 2017, Valerian Saliou <valerian@valeriansaliou.name>
// License: Mozilla Public License v2.0 (MPL v2.0)

use std::thread;
use std::net::TcpListener;

use super::handle::ControlHandle;
use APP_CONF;

pub struct ControlListenBuilder;
pub struct ControlListen;

impl ControlListenBuilder {
    pub fn new() -> ControlListen {
        ControlListen {}
    }
}

impl ControlListen {
    pub fn run(&self) {
        thread::spawn(move || {
            let listener = TcpListener::bind(APP_CONF.control.inet).unwrap();

            for stream in listener.incoming() {
                match stream {
                    Ok(stream) => {
                        thread::spawn(move || {
                            if let Ok(peer_addr) = stream.peer_addr() {
                                debug!("control client connecting: {}", peer_addr);
                            }

                            // Create client
                            ControlHandle::client(stream);
                        });
                    }
                    Err(err) => {
                        warn!("error handling stream: {}", err);
                    }
                }
            }

            info!("listening on tcp://{}", APP_CONF.control.inet);
        });
    }
}
