// Bloom
//
// HTTP REST API caching middleware
// Copyright: 2017, Valerian Saliou <valerian@valeriansaliou.name>
// License: Mozilla Public License v2.0 (MPL v2.0)

use crate::APP_CONF;
use hyper::header::{parsing, Formatter, Header, Raw};
use hyper::Result;
use std::fmt;

#[derive(Clone)]
pub struct HeaderRequestBloomRequestShard(pub u8);

impl Header for HeaderRequestBloomRequestShard {
    fn header_name() -> &'static str {
        "Bloom-Request-Shard"
    }

    fn parse_header(raw: &Raw) -> Result<HeaderRequestBloomRequestShard> {
        parsing::from_one_raw_str(raw)
            .map(HeaderRequestBloomRequestShard)
            .or_else(|e| match APP_CONF.server.default_shard {
                Some(default) => Ok(HeaderRequestBloomRequestShard(default)),
                None => Err(e),
            })
    }

    fn fmt_header(&self, f: &mut Formatter) -> fmt::Result {
        f.fmt_line(self)
    }
}

impl fmt::Display for HeaderRequestBloomRequestShard {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        fmt::Display::fmt(&self.0, f)
    }
}
