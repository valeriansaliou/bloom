// Bloom
//
// HTTP REST API caching middleware
// Copyright: 2017, Valerian Saliou <valerian@valeriansaliou.name>
// License: Mozilla Public License v2.0 (MPL v2.0)

enum HeaderStatus {
    Hit,
    Miss,
    Direct
}

impl HeaderStatus {
    fn to_str(&self) -> &str {
        match *self {
            HeaderStatus::Hit => "HIT",
            HeaderStatus::Miss => "MISS",
            HeaderStatus::Direct => "DIRECT",
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn it_matches_status_string() {
        assert_eq!(HeaderStatus::Hit.to_str(), "HIT");
        assert_eq!(HeaderStatus::Miss.to_str(), "MISS");
        assert_eq!(HeaderStatus::Direct.to_str(), "DIRECT");
    }
}
