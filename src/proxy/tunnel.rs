// Bloom
//
// HTTP REST API caching middleware
// Copyright: 2017, Valerian Saliou <valerian@valeriansaliou.name>
// License: Mozilla Public License v2.0 (MPL v2.0)

use futures::{future, Future, BoxFuture};
use hyper::{Error, Client, Method, Uri, Headers, Body, Request};
use hyper::client::{HttpConnector, Response};

use server::listen::LISTEN_REMOTE;
use APP_CONF;

const MAX_SHARDS: u8 = 1;

lazy_static! {
    static ref SHARD_URI: Uri = format!("http://{}:{}", APP_CONF.proxy.inet.ip(),
        APP_CONF.proxy.inet.port()).parse().unwrap();
}

thread_local! {
    static TUNNEL_CLIENT: Client<HttpConnector> = Client::new(&LISTEN_REMOTE.lock().unwrap()
        .get_mut().clone().unwrap().handle().unwrap());
}

pub struct ProxyTunnelBuilder;

pub struct ProxyTunnel {
    shards: [Option<&'static Uri>; MAX_SHARDS as usize],
}

pub type ProxyTunnelFuture = BoxFuture<Response, Error>;

impl ProxyTunnelBuilder {
    pub fn new() -> ProxyTunnel {
        // We support only 1 shard for now.
        ProxyTunnel {
            shards: [Some(&*SHARD_URI)],
        }
    }
}

impl ProxyTunnel {
    pub fn run(
        &mut self,
        method: &Method,
        uri: &Uri,
        headers: &Headers,
        body: Body,
        shard: u8,
    ) -> ProxyTunnelFuture {
        if shard < MAX_SHARDS {
            // Route to target shard
            // match self.shards[shard as usize] {
            //     Some(ref shard_uri) => {
            //         match format!("{}{}", shard_uri, uri.path()).parse() {
            //             Ok(tunnel_uri) => {
            //                 let mut tunnel_req = Request::new(method.to_owned(), tunnel_uri);

            //                 // Forward headers
            //                 {
            //                     let tunnel_headers = tunnel_req.headers_mut();

            //                     tunnel_headers.clone_from(headers);
            //                 }

            //                 // Forward body?
            //                 match method {
            //                     &Method::Post | &Method::Patch | &Method::Put => {
            //                         // TODO: blocking if non-empty, eg. if PATCH, why?
            //                         tunnel_req.set_body(body);
            //                     }
            //                     _ => {}
            //                 }

            //                 TUNNEL_CLIENT.with(|client| client.request(tunnel_req))
            //             }
            //             Err(err) => future::err(Error::Uri(err)).boxed(),
            //         }
            //     }
            //     None => future::err(Error::Header).boxed(),
            // }

            // TODO
            future::err(Error::Header).boxed()
        } else {
            // Shard out of bounds
            future::err(Error::Header).boxed()
        }
    }
}
