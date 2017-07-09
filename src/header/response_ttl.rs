// Bloom
//
// HTTP REST API caching middleware
// Copyright: 2017, Valerian Saliou <valerian@valeriansaliou.name>
// License: Mozilla Public License v2.0 (MPL v2.0)

use std::fmt;

use hyper::Result;
use hyper::header::{Header, Raw, Formatter, parsing};

#[derive(Clone)]
pub struct HeaderResponseBloomResponseTTL(pub u32);

impl Header for HeaderResponseBloomResponseTTL {
    fn header_name() -> &'static str {
        static NAME: &'static str = "Bloom-Response-TTL";
        NAME
    }

    fn parse_header(raw: &Raw) -> Result<HeaderResponseBloomResponseTTL> {
        parsing::from_one_raw_str(raw).map(HeaderResponseBloomResponseTTL)
    }

    fn fmt_header(&self, f: &mut Formatter) -> fmt::Result {
        f.fmt_line(self)
    }
}

impl fmt::Display for HeaderResponseBloomResponseTTL {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        fmt::Display::fmt(&self.0, f)
    }
}
