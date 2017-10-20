// Bloom
//
// HTTP REST API caching middleware
// Copyright: 2017, Valerian Saliou <valerian@valeriansaliou.name>
// License: Mozilla Public License v2.0 (MPL v2.0)

use std::net::SocketAddr;

pub fn server_log_level() -> String {
    "warn".to_string()
}

pub fn server_inet() -> SocketAddr {
    "[::1]:8080".parse().unwrap()
}

pub fn control_inet() -> SocketAddr {
    "[::1]:8811".parse().unwrap()
}

pub fn control_tcp_timeout() -> u64 {
    300
}

pub fn proxy_shard_shard() -> u8 {
    0
}

pub fn proxy_shard_host() -> String {
    "localhost".to_string()
}

pub fn proxy_shard_port() -> u16 {
    3000
}

pub fn cache_ttl_default() -> usize {
    600
}

pub fn cache_executor_pool() -> u16 {
    64
}

pub fn cache_disable_read() -> bool {
    false
}

pub fn cache_disable_write() -> bool {
    false
}

pub fn redis_host() -> String {
    "localhost".to_string()
}

pub fn redis_port() -> u16 {
    6379
}

pub fn redis_database() -> u8 {
    0
}

pub fn redis_pool_size() -> u32 {
    80
}

pub fn redis_idle_timeout_seconds() -> u64 {
    600
}

pub fn redis_connection_timeout_seconds() -> u64 {
    1
}

pub fn redis_max_key_size() -> usize {
    256000
}

pub fn redis_max_key_expiration() -> usize {
    2592000
}
