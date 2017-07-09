// Bloom
//
// HTTP REST API caching middleware
// Copyright: 2017, Valerian Saliou <valerian@valeriansaliou.name>
// License: Mozilla Public License v2.0 (MPL v2.0)

use hyper::header;
use hyper::header::HeaderView;

pub struct HeaderJanitor;

impl HeaderJanitor {
    pub fn is_contextual(header: &HeaderView) -> bool {
        header.is::<header::Connection>() ||
            header.is::<header::Date>() ||
            header.is::<header::Upgrade>() ||
            header.is::<header::Cookie>()
    }
}
