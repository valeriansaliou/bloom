// Bloom
//
// HTTP REST API caching middleware
// Copyright: 2017, Valerian Saliou <valerian@valeriansaliou.name>
// License: Mozilla Public License v2.0 (MPL v2.0)

use std::fmt;
use hyper::{Result, Error};
use hyper::header::{Header, Raw, Formatter};

#[derive(Clone)]
pub struct HeaderResponseBloomResponseIgnore();

impl Header for HeaderResponseBloomResponseIgnore {
    fn header_name() -> &'static str {
        "Bloom-Response-Ignore"
    }

    fn parse_header(raw: &Raw) -> Result<HeaderResponseBloomResponseIgnore> {
        if raw.eq("1") == true {
            return Ok(HeaderResponseBloomResponseIgnore());
        }
        Err(Error::Header)
    }

    fn fmt_header(&self, f: &mut Formatter) -> fmt::Result {
        f.fmt_line(self)
    }
}

impl fmt::Display for HeaderResponseBloomResponseIgnore {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        fmt::Display::fmt(&1, f)
    }
}
