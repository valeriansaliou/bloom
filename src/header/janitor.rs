// Bloom
//
// HTTP REST API caching middleware
// Copyright: 2017, Valerian Saliou <valerian@valeriansaliou.name>
// License: Mozilla Public License v2.0 (MPL v2.0)

use http::header::{HeaderMap, HeaderName, CONNECTION, COOKIE, DATE, UPGRADE};

use super::response_buckets;
use super::response_ignore;
use super::response_ttl;

pub struct HeaderJanitor;

impl HeaderJanitor {
    pub fn clean(headers: &mut HeaderMap) {
        let headers_to_remove: Vec<HeaderName> = headers
            .keys()
            .filter(|name| Self::is_contextual(name) || Self::is_internal(name))
            .cloned()
            .collect();

        for name in headers_to_remove {
            headers.remove(&name);
        }
    }

    pub fn is_contextual(name: &HeaderName) -> bool {
        name == CONNECTION || name == DATE || name == UPGRADE || name == COOKIE
    }

    pub fn is_internal(name: &HeaderName) -> bool {
        let name_str = name.as_str();
        name_str == response_buckets::HEADER_NAME
            || name_str == response_ignore::HEADER_NAME
            || name_str == response_ttl::HEADER_NAME
    }

    pub fn is_contextual_str(name: &str) -> bool {
        let lower = name.to_lowercase();
        lower == "connection" || lower == "date" || lower == "upgrade" || lower == "cookie"
    }
}
