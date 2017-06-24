// Bloom
//
// HTTP REST API caching middleware
// Copyright: 2017, Valerian Saliou <valerian@valeriansaliou.name>
// License: Mozilla Public License v2.0 (MPL v2.0)

extern crate ini;

use std::net::IpAddr;
use std::net::SocketAddr;

use self::ini::Ini;

use super::defaults;
use super::config::Config;
use super::config::ConfigListen;
use super::config::ConfigMemcached;

pub struct ReaderBuilder;
pub struct Reader;
struct ReaderGetter;

static MODULE: &'static str = "config:reader";

impl ReaderBuilder {
    pub fn new() -> Reader {
        Reader {}
    }
}

impl Reader {
    pub fn read(&self, path: &'static str) -> Config {
        debug!("[{}] reading config file: {}", MODULE, path);

        let conf = Ini::load_from_file(path).unwrap();

        debug!("[{}] read config file: {}", MODULE, path);

        self.make(&conf)
    }

    fn make(&self, conf: &Ini) -> Config {
        Config {
            listen: ConfigListen {
                inet: ReaderGetter::get_inet(&conf, "listen", "inet",
                "host", "port", defaults::LISTEN_HOST,
                defaults::LISTEN_PORT)
            },

            memcached: ConfigMemcached {
                inet: ReaderGetter::get_inet(&conf, "memcached", "inet",
                "host", "port", defaults::MEMCACHED_HOST,
                defaults::MEMCACHED_PORT),

                max_key_size: ReaderGetter::get_u32(&conf, "memcached",
                "max_key_size", defaults::MEMCACHED_MAX_KEY_SIZE),

                max_key_expiration: ReaderGetter::get_u32(&conf, "memcached",
                "max_key_expiration", defaults::MEMCACHED_MAX_KEY_EXPIRATION),

                pool_size: ReaderGetter::get_u8(&conf, "memcached",
                "pool_size", defaults::MEMCACHED_POOL_SIZE),

                reconnect: ReaderGetter::get_u16(&conf, "memcached",
                "reconnect", defaults::MEMCACHED_RECONNECT),

                timeout: ReaderGetter::get_u16(&conf, "memcached",
                "timeout", defaults::MEMCACHED_TIMEOUT)
            }
        }
    }
}

impl ReaderGetter {
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

        debug!("[{}] parsed @{}:{} <inet> => {}", MODULE, group, key,
            value_inet);

        value_inet
    }

    fn get_u8(
        conf: &Ini, group: &'static str, key: &'static str,
        default: &'static str
    ) -> u8 {
        let value_u8 = (*conf).get_from_or(Some(group), key,
            default).parse::<u8>().unwrap();

        debug!("[{}] parsed @{}:{} <u8> => {}", MODULE, group, key, value_u8);

        value_u8
    }

    fn get_u16(
        conf: &Ini, group: &'static str, key: &'static str,
        default: &'static str
    ) -> u16 {
        let value_u16 = (*conf).get_from_or(Some(group), key,
            default).parse::<u16>().unwrap();

        debug!("[{}] parsed @{}:{} <u16> => {}", MODULE, group, key, value_u16);

        value_u16
    }

    fn get_u32(
        conf: &Ini, group: &'static str, key: &'static str,
        default: &'static str
    ) -> u32 {
        let value_u32 = (*conf).get_from_or(Some(group), key,
            default).parse::<u32>().unwrap();

        debug!("[{}] parsed @{}:{} <u32> => {}", MODULE, group, key, value_u32);

        value_u32
    }
}
