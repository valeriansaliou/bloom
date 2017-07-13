// Bloom
//
// HTTP REST API caching middleware
// Copyright: 2017, Valerian Saliou <valerian@valeriansaliou.name>
// License: Mozilla Public License v2.0 (MPL v2.0)

use hyper::header;
use hyper::header::HeaderView;

use super::response_bucket::HeaderResponseBloomResponseBucket;
use super::response_ignore::HeaderResponseBloomResponseIgnore;
use super::response_ttl::HeaderResponseBloomResponseTTL;

pub struct HeaderJanitor;

impl HeaderJanitor {
    pub fn is_contextual(header: &HeaderView) -> bool {
        header.is::<header::Connection>() ||
            header.is::<header::Date>() ||
            header.is::<header::Upgrade>() ||
            header.is::<header::Cookie>()
    }

    pub fn is_internal(header: &HeaderView) -> bool {
        header.is::<HeaderResponseBloomResponseBucket>() ||
            header.is::<HeaderResponseBloomResponseIgnore>() ||
            header.is::<HeaderResponseBloomResponseTTL>()
    }
}
