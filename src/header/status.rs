// Bloom
//
// HTTP REST API caching middleware
// Copyright: 2017, Valerian Saliou <valerian@valeriansaliou.name>
// License: Mozilla Public License v2.0 (MPL v2.0)

use hyper::header::{Formatter, Header, Raw};
use hyper::{Error, Result};
use std::fmt;
use std::str;

#[derive(Clone)]
pub enum HeaderBloomStatusValue {
    Hit,
    Miss,
    Direct,
    Reject,
    Offline,
}

#[derive(Clone)]
pub struct HeaderBloomStatus(pub HeaderBloomStatusValue);

impl HeaderBloomStatusValue {
    const fn to_str(&self) -> &str {
        match *self {
            Self::Hit => "HIT",
            Self::Miss => "MISS",
            Self::Direct => "DIRECT",
            Self::Reject => "REJECT",
            Self::Offline => "OFFLINE",
        }
    }
}

impl Header for HeaderBloomStatus {
    fn header_name() -> &'static str {
        "Bloom-Status"
    }

    fn parse_header(raw: &Raw) -> Result<Self> {
        match raw.one() {
            Some(header_raw) => match str::from_utf8(header_raw) {
                Ok("HIT") => Ok(Self(HeaderBloomStatusValue::Hit)),
                Ok("MISS") => Ok(Self(HeaderBloomStatusValue::Miss)),
                Ok("DIRECT") => Ok(Self(HeaderBloomStatusValue::Direct)),
                Ok("REJECT") => Ok(Self(HeaderBloomStatusValue::Reject)),
                Ok("OFFLINE") => Ok(Self(HeaderBloomStatusValue::Offline)),
                _ => Err(Error::Header),
            },
            _ => Err(Error::Header),
        }
    }

    fn fmt_header(&self, f: &mut Formatter) -> fmt::Result {
        f.fmt_line(self)
    }
}

impl fmt::Display for HeaderBloomStatus {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        fmt::Display::fmt(&self.0.to_str(), f)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn it_matches_status_string() {
        assert_eq!(HeaderBloomStatusValue::Hit.to_str(), "HIT");
        assert_eq!(HeaderBloomStatusValue::Miss.to_str(), "MISS");
        assert_eq!(HeaderBloomStatusValue::Direct.to_str(), "DIRECT");
        assert_eq!(HeaderBloomStatusValue::Reject.to_str(), "REJECT");
        assert_eq!(HeaderBloomStatusValue::Offline.to_str(), "OFFLINE");
    }
}
