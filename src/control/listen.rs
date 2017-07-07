// Bloom
//
// HTTP REST API caching middleware
// Copyright: 2017, Valerian Saliou <valerian@valeriansaliou.name>
// License: Mozilla Public License v2.0 (MPL v2.0)

use std::sync::Arc;
use std::thread;
use std::time::Duration;
use std::net::TcpListener;

use super::handle::ControlHandle;
use config::config::ConfigControl;

pub struct ControlListenBuilder;
pub struct ControlListen {
    config_control: ConfigControl
}

impl ControlListenBuilder {
    pub fn new(config_control: ConfigControl) -> ControlListen {
        ControlListen {
            config_control: config_control
        }
    }
}

impl ControlListen {
    pub fn run(&self) {
        let addr = self.config_control.inet;

        let tcp_timeout = self.config_control.tcp_timeout;

        thread::spawn(move || {
            let listener = TcpListener::bind(addr).unwrap();

            for stream in listener.incoming() {
                match stream {
                    Ok(stream) => {
                        thread::spawn(move || {
                            debug!("control client connecting: {}",
                                stream.peer_addr().unwrap());

                            // Configure stream
                            assert!(stream.set_read_timeout(Some(Duration::new(
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
