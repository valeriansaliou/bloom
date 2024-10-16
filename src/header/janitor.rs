// Bloom
//
// HTTP REST API caching middleware
// Copyright: 2017, Valerian Saliou <valerian@valeriansaliou.name>
// License: Mozilla Public License v2.0 (MPL v2.0)

use hyper::header::HeaderView;
use hyper::{header, Headers};

use super::response_buckets::HeaderResponseBloomResponseBuckets;
use super::response_ignore::HeaderResponseBloomResponseIgnore;
use super::response_ttl::HeaderResponseBloomResponseTTL;

pub struct HeaderJanitor;

impl HeaderJanitor {
    pub fn clean(headers: &mut Headers) {
        // Map headers to clean-up
        let mut headers_remove: Vec<String> = Vec::new();

        for header_view in headers.iter() {
            // Do not forward contextual and internal headers (ie. 'Bloom-Response-*' headers)
            if Self::is_contextual(&header_view) || Self::is_internal(&header_view) {
                headers_remove.push(String::from(header_view.name()));
            }
        }

        // Proceed headers clean-up
        for header_remove in &headers_remove {
            headers.remove_raw(header_remove.as_ref());
        }
    }

    pub fn is_contextual(header: &HeaderView) -> bool {
        header.is::<header::Connection>()
            || header.is::<header::Date>()
            || header.is::<header::Upgrade>()
            || header.is::<header::Cookie>()
    }

    pub fn is_internal(header: &HeaderView) -> bool {
        header.is::<HeaderResponseBloomResponseBuckets>()
            || header.is::<HeaderResponseBloomResponseIgnore>()
            || header.is::<HeaderResponseBloomResponseTTL>()
    }
}
