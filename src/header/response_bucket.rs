// Bloom
//
// HTTP REST API caching middleware
// Copyright: 2017, Valerian Saliou <valerian@valeriansaliou.name>
// License: Mozilla Public License v2.0 (MPL v2.0)

use std::fmt;
use std::str;

use hyper::{Result, Error};
use hyper::header::{Header, Raw, Formatter, parsing};

#[derive(Clone)]
pub struct HeaderResponseBloomResponseBucket(pub String);

impl Header for HeaderResponseBloomResponseBucket {
    fn header_name() -> &'static str {
        static NAME: &'static str = "Bloom-Response-Bucket";
        NAME
    }

    fn parse_header(raw: &Raw) -> Result<HeaderResponseBloomResponseBucket> {
        parsing::from_one_raw_str(raw).map(HeaderResponseBloomResponseBucket)
    }

    fn fmt_header(&self, f: &mut Formatter) -> fmt::Result {
        f.fmt_line(self)
    }
}

impl fmt::Display for HeaderResponseBloomResponseBucket {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        fmt::Display::fmt(&self.0, f)
    }
}
