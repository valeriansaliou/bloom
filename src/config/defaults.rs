// Bloom
//
// HTTP REST API caching middleware
// Copyright: 2017, Valerian Saliou <valerian@valeriansaliou.name>
// License: Mozilla Public License v2.0 (MPL v2.0)

pub static SERVER_HOST: &'static str = "::1";
pub static SERVER_PORT: &'static str = "80";

pub static CONTROL_HOST: &'static str = "::1";
pub static CONTROL_PORT: &'static str = "811";
pub static CONTROL_TCP_TIMEOUT: &'static str = "300";

pub static PROXY_SHARD: &'static str = "0";
pub static PROXY_HOST: &'static str = "::1";
pub static PROXY_PORT: &'static str = "3000";
pub static PROXY_TUNNEL_THREADS: &'static str = "2";

pub static CACHE_TTL_DEFAULT: &'static str = "600";

pub static REDIS_HOST: &'static str = "127.0.0.1";
pub static REDIS_PORT: &'static str = "6379";
pub static REDIS_DATABASE: &'static str = "0";
pub static REDIS_MAX_KEY_SIZE: &'static str = "256000";
pub static REDIS_MAX_KEY_EXPIRATION: &'static str = "2592000";
