// Bloom
//
// HTTP REST API caching middleware
// Copyright: 2017, Valerian Saliou <valerian@valeriansaliou.name>
// License: Mozilla Public License v2.0 (MPL v2.0)

use farmhash;
use http::header::HeaderMap;
use http::{Method, StatusCode, Version};

use super::check::CacheCheck;
use super::route::CacheRoute;
use crate::header::janitor::HeaderJanitor;
use crate::header::response_buckets;
use crate::header::response_ttl;
use crate::APP_CACHE_STORE;
use crate::APP_CONF;

pub struct CacheWrite;

pub struct CacheWriteResult {
    pub body: Result<String, Option<String>>,
    pub fingerprint: Option<String>,
    pub status: StatusCode,
    pub headers: HeaderMap,
}


impl CacheWrite {
    pub async fn save(
        key: String,
        key_mask: String,
        auth_hash: String,
        shard: u8,
        method: Method,
        version: Version,
        status: StatusCode,
        mut headers: HeaderMap,
        body_bytes: bytes::Bytes,
    ) -> CacheWriteResult {
        let body_result = String::from_utf8(body_bytes.to_vec());

        match body_result {
            Ok(body_value) => {
                debug!("checking whether to write cache for key: {}", &key);

                if APP_CONF.cache.disable_write == false
                    && CacheCheck::from_response(&method, &status, &headers) == true
                {
                    debug!("key: {} cacheable, writing cache", &key);

                    let mut key_tags: Vec<(String, String)> =
                        match headers.get(response_buckets::HEADER_NAME) {
                            None => Vec::new(),
                            Some(value) => {
                                if let Ok(v) = value.to_str() {
                                    response_buckets::parse_buckets(v)
                                        .iter()
                                        .map(|bucket| {
                                            CacheRoute::gen_key_bucket_from_hash(
                                                shard,
                                                &CacheRoute::hash(bucket),
                                            )
                                        })
                                        .collect()
                                } else {
                                    Vec::new()
                                }
                            }
                        };

                    key_tags.push(CacheRoute::gen_key_auth_from_hash(shard, &auth_hash));

                    let ttl = match headers.get(response_ttl::HEADER_NAME) {
                        None => APP_CONF.cache.ttl_default,
                        Some(value) => {
                            if let Ok(v) = value.to_str() {
                                response_ttl::parse_ttl(v).unwrap_or(APP_CONF.cache.ttl_default)
                            } else {
                                APP_CONF.cache.ttl_default
                            }
                        }
                    };

                    HeaderJanitor::clean(&mut headers);

                    let body_string = format!(
                        "{}\n{}\n{}",
                        CacheWrite::generate_chain_banner(&version, &status),
                        CacheWrite::generate_chain_headers(&headers),
                        body_value
                    );

                    let fingerprint = Self::process_body_fingerprint(&body_string);

                    match APP_CACHE_STORE
                        .set(key, key_mask, body_string, fingerprint.clone(), ttl, key_tags)
                        .await
                    {
                        Ok(_) => {
                            debug!("wrote cache");

                            CacheWriteResult {
                                body: Ok(body_value),
                                fingerprint: Some(fingerprint),
                                status,
                                headers,
                            }
                        }
                        Err(err) => {
                            warn!("could not write cache because: {:?}", err);

                            CacheWriteResult {
                                body: Err(Some(body_value)),
                                fingerprint: Some(fingerprint),
                                status,
                                headers,
                            }
                        }
                    }
                } else {
                    debug!("key: {} not cacheable, ignoring", &key);

                    CacheWriteResult {
                        body: Err(Some(body_value)),
                        fingerprint: None,
                        status,
                        headers,
                    }
                }
            }
            Err(_) => {
                error!("failed unwrapping body value for key: {}, ignoring", &key);

                CacheWriteResult {
                    body: Err(None),
                    fingerprint: None,
                    status,
                    headers,
                }
            }
        }
    }

    fn generate_chain_banner(version: &Version, status: &StatusCode) -> String {
        let version_str = match version {
            &Version::HTTP_09 => "HTTP/0.9",
            &Version::HTTP_10 => "HTTP/1.0",
            &Version::HTTP_11 => "HTTP/1.1",
            &Version::HTTP_2 => "HTTP/2",
            &Version::HTTP_3 => "HTTP/3",
            _ => "HTTP/1.1",
        };
        format!("{} {}", version_str, status)
    }

    fn generate_chain_headers(headers: &HeaderMap) -> String {
        headers
            .iter()
            .filter(|(name, _)| HeaderJanitor::is_contextual_str(name.as_str()) == false)
            .map(|(name, value)| {
                format!(
                    "{}: {}\n",
                    name.as_str(),
                    value.to_str().unwrap_or("")
                )
            })
            .collect()
    }

    fn process_body_fingerprint(body_string: &str) -> String {
        format!("{:x}", farmhash::fingerprint64(body_string.as_bytes()))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use bytes::Bytes;

    #[tokio::test]
    async fn it_saves_cache_with_empty_body() {
        let result = CacheWrite::save(
            "bloom:0:c:90d52bc6:f773d6f1".to_string(),
            "90d52bc6:f773d6f1".to_string(),
            "90d52bc6".to_string(),
            0,
            Method::GET,
            Version::HTTP_11,
            StatusCode::OK,
            HeaderMap::new(),
            Bytes::new(),
        )
        .await;

        assert!(result.body.is_ok() || result.body.is_err());
    }
}
