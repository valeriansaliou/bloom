// Bloom
//
// HTTP REST API caching middleware
// Copyright: 2017, Valerian Saliou <valerian@valeriansaliou.name>
// License: Mozilla Public License v2.0 (MPL v2.0)

use http_body_util::Full;
use hyper::body::{Bytes, Incoming};
use hyper::{Request, Response};
use std::convert::Infallible;

use crate::proxy::serve::ProxyServe;

pub type BoxBody = Full<Bytes>;

pub async fn handle_request(
    req: Request<Incoming>,
) -> Result<Response<BoxBody>, Infallible> {
    debug!("called proxy serve");

    Ok(ProxyServe::handle(req).await)
}
