// Bloom
//
// HTTP REST API caching middleware
// Copyright: 2017, Valerian Saliou <valerian@valeriansaliou.name>
// License: Mozilla Public License v2.0 (MPL v2.0)

use std::net::SocketAddr;

pub fn server_log_level() -> String {
    "error".to_string()
}

pub fn server_inet() -> SocketAddr {
    "[::1]:8080".parse().unwrap()
}

pub fn control_inet() -> SocketAddr {
    "[::1]:8811".parse().unwrap()
}

pub const fn control_tcp_timeout() -> u64 {
    300
}

pub const fn proxy_shard_default() -> u8 {
    0
}

pub const fn proxy_shard_shard() -> u8 {
    0
}

pub fn proxy_shard_host() -> String {
    "localhost".to_string()
}

pub const fn proxy_shard_port() -> u16 {
    3000
}

pub const fn cache_ttl_default() -> usize {
    600
}

pub const fn cache_executor_pool() -> u16 {
    64
}

pub const fn cache_disable_read() -> bool {
    false
}

pub const fn cache_disable_write() -> bool {
    false
}

pub const fn cache_compress_body() -> bool {
    true
}
pub fn redis_host() -> String {
    "localhost".to_string()
}

pub const fn redis_port() -> u16 {
    6379
}

pub const fn redis_database() -> u8 {
    0
}

pub const fn redis_pool_size() -> u32 {
    80
}

pub const fn redis_max_lifetime_seconds() -> u64 {
    60
}

pub const fn redis_idle_timeout_seconds() -> u64 {
    600
}

pub const fn redis_connection_timeout_seconds() -> u64 {
    1
}

pub const fn redis_max_key_size() -> usize {
    256_000
}

pub const fn redis_max_key_expiration() -> usize {
    2_592_000
}
