// Bloom
//
// HTTP REST API caching middleware
// Copyright: 2017, Valerian Saliou <valerian@valeriansaliou.name>
// License: Mozilla Public License v2.0 (MPL v2.0)

use std::str::FromStr;
use std::fmt::Debug;
use std::fmt::Display;
use std::net::IpAddr;
use std::net::SocketAddr;
use ini::Ini;

use super::defaults;
use super::config::*;
use APP_ARGS;

pub struct ConfigReader;
struct ConfigReaderGetter;

impl ConfigReader {
    pub fn make() -> Config {
        debug!("reading config file: {}", &APP_ARGS.config);

        let conf = Ini::load_from_file(&APP_ARGS.config).unwrap();

        debug!("read config file: {}", &APP_ARGS.config);

        Config {
            server: ConfigServer {
                log_level: ConfigReaderGetter::get_generic(
                    &conf,
                    "server",
                    "log_level",
                    defaults::SERVER_LOG_LEVEL,
                ),

                inet: ConfigReaderGetter::get_inet(
                    &conf,
                    "server",
                    "inet",
                    "host",
                    "port",
                    defaults::SERVER_HOST,
                    defaults::SERVER_PORT,
                ),
            },

            control: ConfigControl {
                inet: ConfigReaderGetter::get_inet(
                    &conf,
                    "control",
                    "inet",
                    "host",
                    "port",
                    defaults::CONTROL_HOST,
                    defaults::CONTROL_PORT,
                ),

                tcp_timeout: ConfigReaderGetter::get_generic(
                    &conf,
                    "control",
                    "tcp_timeout",
                    defaults::CONTROL_TCP_TIMEOUT,
                ),
            },

            proxy: ConfigProxy {
                shard: ConfigReaderGetter::get_generic(
                    &conf,
                    "proxy",
                    "shard",
                    defaults::PROXY_SHARD,
                ),

                inet: ConfigReaderGetter::get_inet(
                    &conf,
                    "proxy",
                    "inet",
                    "host",
                    "port",
                    defaults::PROXY_HOST,
                    defaults::PROXY_PORT,
                ),
            },

            cache: ConfigCache {
                ttl_default: ConfigReaderGetter::get_generic(
                    &conf,
                    "cache",
                    "ttl_default",
                    defaults::CACHE_TTL_DEFAULT,
                ),
            },

            redis: ConfigRedis {
                inet: ConfigReaderGetter::get_inet(
                    &conf,
                    "redis",
                    "inet",
                    "host",
                    "port",
                    defaults::REDIS_HOST,
                    defaults::REDIS_PORT,
                ),

                database: ConfigReaderGetter::get_generic(
                    &conf,
                    "redis",
                    "database",
                    defaults::REDIS_DATABASE,
                ),

                pool_size: ConfigReaderGetter::get_generic(
                    &conf,
                    "redis",
                    "pool_size",
                    defaults::REDIS_POOL_SIZE,
                ),

                idle_timeout_seconds: ConfigReaderGetter::get_generic(
                    &conf,
                    "redis",
                    "idle_timeout_seconds",
                    defaults::REDIS_IDLE_TIMEOUT_SECONDS,
                ),

                connection_timeout_seconds: ConfigReaderGetter::get_generic(
                    &conf,
                    "redis",
                    "connection_timeout_seconds",
                    defaults::REDIS_CONNECTION_TIMEOUT_SECONDS,
                ),

                max_key_size: ConfigReaderGetter::get_generic(
                    &conf,
                    "redis",
                    "max_key_size",
                    defaults::REDIS_MAX_KEY_SIZE,
                ),

                max_key_expiration: ConfigReaderGetter::get_generic(
                    &conf,
                    "redis",
                    "max_key_expiration",
                    defaults::REDIS_MAX_KEY_EXPIRATION,
                ),
            },
        }
    }
}

impl ConfigReaderGetter {
    fn get_inet(
        conf: &Ini,
        group: &'static str,
        key: &'static str,
        key_host: &'static str,
        key_port: &'static str,
        default_host: &'static str,
        default_port: &'static str,
    ) -> SocketAddr {
        let value_host = (*conf)
            .get_from_or(Some(group), key_host, default_host)
            .parse::<IpAddr>()
            .unwrap();

        let value_port = (*conf)
            .get_from_or(Some(group), key_port, default_port)
            .parse::<u16>()
            .unwrap();

        let value_inet = SocketAddr::new(value_host, value_port);

        debug!("parsed @{}:{} => {}", group, key, value_inet);

        value_inet
    }

    fn get_generic<T>(
        conf: &Ini,
        group: &'static str,
        key: &'static str,
        default: &'static str,
    ) -> T
    where
        T: Display + FromStr,
        <T as FromStr>::Err: Debug,
    {
        let value = (*conf)
            .get_from_or(Some(group), key, default)
            .parse::<T>()
            .unwrap();

        debug!("parsed @{}:{} => {}", group, key, value);

        value
    }
}
