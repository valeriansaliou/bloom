// Bloom
//
// HTTP REST API caching middleware
// Copyright: 2017, Valerian Saliou <valerian@valeriansaliou.name>
// License: Mozilla Public License v2.0 (MPL v2.0)

use std::fmt;

use hyper::Result;
use hyper::header::{Header, Raw, Formatter, parsing};

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct HeaderRequestBloomRequestShard(pub u8);

impl Header for HeaderRequestBloomRequestShard {
    #[inline]
    fn header_name() -> &'static str {
        static NAME: &'static str = "Bloom-Request-Shard";
        NAME
    }

    fn parse_header(raw: &Raw) -> Result<HeaderRequestBloomRequestShard> {
        parsing::from_one_raw_str(raw).map(HeaderRequestBloomRequestShard)
    }

    #[inline]
    fn fmt_header(&self, f: &mut Formatter) -> fmt::Result {
        f.fmt_line(self)
    }
}

impl fmt::Display for HeaderRequestBloomRequestShard {
    #[inline]
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        fmt::Display::fmt(&self.0, f)
    }
}
