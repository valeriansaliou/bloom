// Bloom
//
// HTTP REST API caching middleware
// Copyright: 2017, Valerian Saliou <valerian@valeriansaliou.name>
// License: Mozilla Public License v2.0 (MPL v2.0)

use std::thread;
use std::time::Duration;
use std::net::TcpListener;

use super::handle::ControlHandle;
use ::APP_CONF;
use config::config::ConfigControl;
use cache::store::CacheStore;

pub struct ControlListenBuilder;
pub struct ControlListen;

impl ControlListenBuilder {
    pub fn new() -> ControlListen {
        ControlListen {}
    }
}

impl ControlListen {
    pub fn run(&self) {
        let addr = APP_CONF.control.inet;

        let tcp_timeout = APP_CONF.control.tcp_timeout;

        thread::spawn(move || {
            let listener = TcpListener::bind(addr).unwrap();

            for stream in listener.incoming() {
                match stream {
                    Ok(stream) => {
                        thread::spawn(move || {
                            debug!("control client connecting: {}",
                                stream.peer_addr().unwrap());

                            // Configure stream
                            assert!(stream.set_nodelay(true).is_ok());
                            assert!(stream.set_write_timeout(Some(Duration::new(
                                tcp_timeout, 0))).is_ok());
                            assert!(stream.set_write_timeout(Some(Duration::new(
                                tcp_timeout, 0))).is_ok());

                            // Create client
                            ControlHandle::client(stream);
                        });
                    }
                    Err(err) => {
                        warn!("error handling stream: {}", err);
                    }
                }
            }

            info!("listening on tcp://{}", addr);
        });
    }
}
