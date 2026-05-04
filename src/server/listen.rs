// Bloom
//
// HTTP REST API caching middleware
// Copyright: 2017, Valerian Saliou <valerian@valeriansaliou.name>
// License: Mozilla Public License v2.0 (MPL v2.0)

use hyper::server::conn::http1;
use hyper::service::service_fn;
use hyper_util::rt::TokioIo;
use std::net::SocketAddr;
use tokio::net::TcpListener;

use super::handle::handle_request;
use crate::APP_CONF;

pub struct ServerListenBuilder;
pub struct ServerListen;

impl ServerListenBuilder {
    pub fn new() -> ServerListen {
        ServerListen {}
    }
}

impl ServerListen {
    pub fn run(&self) {
        let addr: SocketAddr = APP_CONF.server.inet;

        let runtime = tokio::runtime::Runtime::new().expect("failed to create tokio runtime");

        runtime.block_on(async move {
            let listener = TcpListener::bind(addr)
                .await
                .expect("failed to bind server");

            info!("listening on http://{}", addr);

            loop {
                match listener.accept().await {
                    Ok((stream, _remote_addr)) => {
                        let io = TokioIo::new(stream);

                        tokio::spawn(async move {
                            let service = service_fn(move |req| {
                                let fut = handle_request(req);
                                async move { fut.await }
                            });

                            if let Err(err) = http1::Builder::new()
                                .keep_alive(true)
                                .serve_connection(io, service)
                                .await
                            {
                                debug!("connection error: {:?}", err);
                            }
                        });
                    }
                    Err(err) => {
                        warn!("accept error: {}", err);
                    }
                }
            }
        });
    }
}
