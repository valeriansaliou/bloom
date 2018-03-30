// Bloom
//
// HTTP REST API caching middleware
// Copyright: 2018, Valerian Saliou <valerian@valeriansaliou.name>
// License: Mozilla Public License v2.0 (MPL v2.0)

use std::fmt;
use hyper::Result;
use hyper::header::{Header, Raw, Formatter, parsing};

#[derive(Clone)]
pub struct HeaderBloomRay(String);

impl Header for HeaderBloomRay {
    fn header_name() -> &'static str {
        "Bloom-Ray"
    }

    fn parse_header(raw: &Raw) -> Result<HeaderBloomRay> {
        parsing::from_one_raw_str(raw).map(HeaderBloomRay)
    }

    fn fmt_header(&self, f: &mut Formatter) -> fmt::Result {
        f.fmt_line(self)
    }
}

impl fmt::Display for HeaderBloomRay {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", &self.0)
    }
}
