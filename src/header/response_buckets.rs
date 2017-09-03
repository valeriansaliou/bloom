// Bloom
//
// HTTP REST API caching middleware
// Copyright: 2017, Valerian Saliou <valerian@valeriansaliou.name>
// License: Mozilla Public License v2.0 (MPL v2.0)

use std::fmt;
use hyper::Result;
use hyper::header::{Header, Raw, Formatter, parsing};

#[derive(Clone)]
pub struct HeaderResponseBloomResponseBuckets(pub Vec<String>);

impl Header for HeaderResponseBloomResponseBuckets {
    fn header_name() -> &'static str {
        static NAME: &'static str = "Bloom-Response-Buckets";
        NAME
    }

    fn parse_header(raw: &Raw) -> Result<HeaderResponseBloomResponseBuckets> {
        parsing::from_comma_delimited(raw).map(HeaderResponseBloomResponseBuckets)
    }

    fn fmt_header(&self, f: &mut Formatter) -> fmt::Result {
        f.fmt_line(self)
    }
}

impl fmt::Display for HeaderResponseBloomResponseBuckets {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        parsing::fmt_comma_delimited(f, &self.0)
    }
}
