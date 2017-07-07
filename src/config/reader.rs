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
use super::config::Config;
use super::config::ConfigServer;
use super::config::ConfigControl;
use super::config::ConfigProxy;
use super::config::ConfigMemcached;

pub struct ConfigReaderBuilder;
pub struct ConfigReader;
struct ConfigReaderGetter;

impl ConfigReaderBuilder {
    pub fn new() -> ConfigReader {
        ConfigReader {}
    }
}

impl ConfigReader {
    pub fn read(&self, path: &str) -> Config {
        debug!("reading config file: {}", path);

        let conf = Ini::load_from_file(path).unwrap();

        debug!("read config file: {}", path);

        self.make(&conf)
    }

    fn make(&self, conf: &Ini) -> Config {
        Config {
            server: ConfigServer {
                inet: ConfigReaderGetter::get_inet(&conf, "server", "inet",
                "host", "port", defaults::SERVER_HOST,
                defaults::SERVER_PORT)
            },

            control: ConfigControl {
                inet: ConfigReaderGetter::get_inet(&conf, "control", "inet",
                "host", "port", defaults::CONTROL_HOST,
                defaults::CONTROL_PORT),

                tcp_timeout: ConfigReaderGetter::get_generic(&conf,
                    "control", "tcp_timeout",
                    defaults::CONTROL_TCP_TIMEOUT)
            },

            proxy: ConfigProxy {
                shard: ConfigReaderGetter::get_generic(&conf, "proxy",
                "shard", defaults::PROXY_SHARD),

                inet: ConfigReaderGetter::get_inet(&conf, "proxy", "inet",
                "host", "port", defaults::PROXY_HOST,
                defaults::PROXY_PORT),

                connect_timeout: ConfigReaderGetter::get_generic(&conf, "proxy",
                "connect_timeout", defaults::PROXY_CONNECT_TIMEOUT),

                read_timeout: ConfigReaderGetter::get_generic(&conf, "proxy",
                "read_timeout", defaults::PROXY_READ_TIMEOUT),

                send_timeout: ConfigReaderGetter::get_generic(&conf, "proxy",
                "send_timeout", defaults::PROXY_SEND_TIMEOUT),
            },

            memcached: ConfigMemcached {
                inet: ConfigReaderGetter::get_inet(&conf, "memcached", "inet",
                "host", "port", defaults::MEMCACHED_HOST,
                defaults::MEMCACHED_PORT),

                max_key_size: ConfigReaderGetter::get_generic(&conf,
                    "memcached", "max_key_size",
                    defaults::MEMCACHED_MAX_KEY_SIZE),

                max_key_expiration: ConfigReaderGetter::get_generic(&conf,
                    "memcached", "max_key_expiration",
                    defaults::MEMCACHED_MAX_KEY_EXPIRATION),

                pool_size: ConfigReaderGetter::get_generic(&conf, "memcached",
                "pool_size", defaults::MEMCACHED_POOL_SIZE),

                reconnect: ConfigReaderGetter::get_generic(&conf, "memcached",
                "reconnect", defaults::MEMCACHED_RECONNECT),

                timeout: ConfigReaderGetter::get_generic(&conf, "memcached",
                "timeout", defaults::MEMCACHED_TIMEOUT)
            }
        }
    }
}

impl ConfigReaderGetter {
    fn get_inet(
        conf: &Ini, group: &'static str, key: &'static str,
        key_host: &'static str, key_port: &'static str,
        default_host: &'static str, default_port: &'static str
    ) -> SocketAddr {
        let value_host = (*conf).get_from_or(Some(group), key_host,
            default_host).parse::<IpAddr>().unwrap();

        let value_port = (*conf).get_from_or(Some(group), key_port,
            default_port).parse::<u16>().unwrap();

        let value_inet = SocketAddr::new(value_host, value_port);

        debug!("parsed @{}:{} => {}", group, key,
            value_inet);

        value_inet
    }

    fn get_generic<T>(
        conf: &Ini, group: &'static str, key: &'static str,
        default: &'static str
    ) -> T where T: Display + FromStr, <T as FromStr>::Err: Debug {
        let value = (*conf).get_from_or(Some(group), key,
            default).parse::<T>().unwrap();

        debug!("parsed @{}:{} => {}", group, key, value);

        value
    }
}
