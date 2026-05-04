// Bloom
//
// HTTP REST API caching middleware
// Copyright: 2017, Valerian Saliou <valerian@valeriansaliou.name>
// License: Mozilla Public License v2.0 (MPL v2.0)

use http::header::HeaderValue;
use std::fmt;

pub const HEADER_NAME: &str = "bloom-status";

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
    pub fn to_str(&self) -> &str {
        match *self {
            HeaderBloomStatusValue::Hit => "HIT",
            HeaderBloomStatusValue::Miss => "MISS",
            HeaderBloomStatusValue::Direct => "DIRECT",
            HeaderBloomStatusValue::Reject => "REJECT",
            HeaderBloomStatusValue::Offline => "OFFLINE",
        }
    }
}

impl HeaderBloomStatus {
    pub fn to_header_value(&self) -> HeaderValue {
        match self.0 {
            HeaderBloomStatusValue::Hit => HeaderValue::from_static("HIT"),
            HeaderBloomStatusValue::Miss => HeaderValue::from_static("MISS"),
            HeaderBloomStatusValue::Direct => HeaderValue::from_static("DIRECT"),
            HeaderBloomStatusValue::Reject => HeaderValue::from_static("REJECT"),
            HeaderBloomStatusValue::Offline => HeaderValue::from_static("OFFLINE"),
        }
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
