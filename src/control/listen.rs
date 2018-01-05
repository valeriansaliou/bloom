// Bloom
//
// HTTP REST API caching middleware
// Copyright: 2017, Valerian Saliou <valerian@valeriansaliou.name>
// License: Mozilla Public License v2.0 (MPL v2.0)

use std::thread;
use std::process;
use std::net::TcpListener;

use super::handle::ControlHandle;
use {APP_CONF, THREAD_NAME_CONTROL_MASTER, THREAD_NAME_CONTROL_CLIENT};

pub struct ControlListenBuilder;
pub struct ControlListen;

impl ControlListenBuilder {
    pub fn new() -> ControlListen {
        ControlListen {}
    }
}

impl ControlListen {
    pub fn run(&self) {
        thread::Builder::new()
            .name(THREAD_NAME_CONTROL_MASTER.to_string())
            .spawn(move || {
                match TcpListener::bind(APP_CONF.control.inet) {
                    Ok(listener) => {
                        for stream in listener.incoming() {
                            match stream {
                                Ok(stream) => {
                                    thread::Builder::new()
                                        .name(THREAD_NAME_CONTROL_CLIENT.to_string())
                                        .spawn(move || {
                                            if let Ok(peer_addr) = stream.peer_addr() {
                                                debug!("control client connecting: {}", peer_addr);
                                            }

                                            // Create client
                                            ControlHandle::client(stream);
                                        })
                                        .ok();
                                }
                                Err(err) => {
                                    warn!("error handling stream: {}", err);
                                }
                            }
                        }

                        info!("listening on tcp://{}", APP_CONF.control.inet);
                    }
                    Err(err) => {
                        error!("error binding control listener: {}", err);

                        // Exit Bloom
                        process::exit(1);
                    }
                }
            })
            .ok();
    }
}
