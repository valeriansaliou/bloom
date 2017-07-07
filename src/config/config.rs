// Bloom
//
// HTTP REST API caching middleware
// Copyright: 2017, Valerian Saliou <valerian@valeriansaliou.name>
// License: Mozilla Public License v2.0 (MPL v2.0)

use std::net::SocketAddr;

pub struct Config {
    pub server: ConfigServer,
    pub control: ConfigControl,
    pub proxy: ConfigProxy,
    pub memcached: ConfigMemcached
}

pub struct ConfigServer {
    pub inet: SocketAddr
}

pub struct ConfigControl {
    pub inet: SocketAddr,
    pub tcp_read_timeout: u64,
    pub tcp_write_timeout: u64
}

pub struct ConfigProxy {
    pub shard: u8,
    pub inet: SocketAddr,
    pub connect_timeout: u16,
    pub read_timeout: u16,
    pub send_timeout: u16
}

pub struct ConfigMemcached {
    pub inet: SocketAddr,
    pub max_key_size: usize,
    pub max_key_expiration: u32,
    pub pool_size: u8,
    pub reconnect: u16,
    pub timeout: u16
}
