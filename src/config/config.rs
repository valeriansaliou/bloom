// Bloom
//
// HTTP REST API caching middleware
// Copyright: 2017, Valerian Saliou <valerian@valeriansaliou.name>
// License: Mozilla Public License v2.0 (MPL v2.0)

use std::net::SocketAddr;

use super::defaults;

#[derive(Deserialize)]
pub struct Config {
    pub server: ConfigServer,
    pub control: ConfigControl,
    pub proxy: ConfigProxy,
    pub cache: ConfigCache,
    pub redis: ConfigRedis,
}

#[derive(Deserialize)]
pub struct ConfigServer {
    #[serde(default = "defaults::server_log_level")]
    pub log_level: String,

    #[serde(default = "defaults::server_inet")]
    pub inet: SocketAddr,
}

#[derive(Deserialize)]
pub struct ConfigControl {
    #[serde(default = "defaults::control_inet")]
    pub inet: SocketAddr,

    #[serde(default = "defaults::control_tcp_timeout")]
    pub tcp_timeout: u64,
}

#[derive(Deserialize)]
pub struct ConfigProxy {
    #[serde(default = "defaults::proxy_shard_default")]
    pub shard_default: u8,

    pub shard: Vec<ConfigProxyShard>,
}

#[derive(Deserialize)]
pub struct ConfigProxyShard {
    #[serde(default = "defaults::proxy_shard_shard")]
    pub shard: u8,

    #[serde(default = "defaults::proxy_shard_host")]
    pub host: String,

    #[serde(default = "defaults::proxy_shard_port")]
    pub port: u16,
}

#[derive(Deserialize)]
pub struct ConfigCache {
    #[serde(default = "defaults::cache_ttl_default")]
    pub ttl_default: usize,

    #[serde(default = "defaults::cache_executor_pool")]
    pub executor_pool: u16,

    #[serde(default = "defaults::cache_disable_read")]
    pub disable_read: bool,

    #[serde(default = "defaults::cache_disable_write")]
    pub disable_write: bool,

    #[serde(default = "defaults::cache_compress_body")]
    pub compress_body: bool,
}

#[derive(Deserialize)]
pub struct ConfigRedis {
    #[serde(default = "defaults::redis_host")]
    pub host: String,

    #[serde(default = "defaults::redis_port")]
    pub port: u16,

    pub password: Option<String>,

    #[serde(default = "defaults::redis_database")]
    pub database: u8,

    #[serde(default = "defaults::redis_pool_size")]
    pub pool_size: u32,

    #[serde(default = "defaults::redis_max_lifetime_seconds")]
    pub max_lifetime_seconds: u64,

    #[serde(default = "defaults::redis_idle_timeout_seconds")]
    pub idle_timeout_seconds: u64,

    #[serde(default = "defaults::redis_connection_timeout_seconds")]
    pub connection_timeout_seconds: u64,

    #[serde(default = "defaults::redis_max_key_size")]
    pub max_key_size: usize,

    #[serde(default = "defaults::redis_max_key_expiration")]
    pub max_key_expiration: usize,
}
