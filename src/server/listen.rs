// Bloom
//
// HTTP REST API caching middleware
// Copyright: 2017, Valerian Saliou <valerian@valeriansaliou.name>
// License: Mozilla Public License v2.0 (MPL v2.0)

use hyper::server::conn::http1;
use hyper_util::rt::TokioIo;
use tokio::net::TcpListener;
use tokio::runtime::Runtime;

use super::handle::ServerRequestHandle;
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
        let server_inet = APP_CONF.server.inet;

        Runtime::new()
            .expect("failed to create server runtime")
            .block_on(async {
                let listener = TcpListener::bind(server_inet)
                    .await
                    .expect("failed to bind server tcp listener");

                info!("listening on http://{}", server_inet);

                loop {
                    match listener.accept().await {
                        Ok((stream, _)) => {
                            let io = TokioIo::new(stream);

                            tokio::spawn(async move {
                                if let Err(err) = http1::Builder::new()
                                    .serve_connection(io, ServerRequestHandle)
                                    .await
                                {
                                    debug!("server client connection dropped: {}", err);
                                }
                            });
                        }
                        Err(err) => {
                            error!("server accept error: {}", err);
                        }
                    }
                }
            });
    }
}
