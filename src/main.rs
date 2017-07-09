// Bloom
//
// HTTP REST API caching middleware
// Copyright: 2017, Valerian Saliou <valerian@valeriansaliou.name>
// License: Mozilla Public License v2.0 (MPL v2.0)

#[macro_use] extern crate log;
#[macro_use] extern crate clap;
#[macro_use] extern crate lazy_static;
extern crate ini;
extern crate hyper;
extern crate farmhash;
extern crate futures;
extern crate bmemcached;
extern crate rand;

mod config;
mod header;
mod proxy;
mod cache;
mod control;
mod server;

use clap::{App, Arg};
use config::config::Config;
use config::logger::ConfigLogger;
use config::reader::ConfigReader;
use cache::store::{CacheStore, CacheStoreBuilder};
use proxy::serve::{ProxyServe, ProxyServeBuilder};
use control::listen::ControlListenBuilder;
use server::listen::ServerListenBuilder;

struct AppArgs {
    config: String
}

lazy_static! {
    static ref APP_ARGS: AppArgs = make_app_args();
    static ref APP_CONF: Config = ConfigReader::make();
    static ref APP_CACHE_STORE: CacheStore = CacheStoreBuilder::new();
    static ref APP_PROXY_SERVE: ProxyServe = ProxyServeBuilder::new();
}

fn make_app_args() -> AppArgs {
    let matches = App::new(crate_name!())
        .version(crate_version!())
        .author(crate_authors!("\n"))
        .about(crate_description!())
        .arg(Arg::with_name("config")
            .short("c")
            .long("config")
            .help("Path to configuration file")
            .default_value("./config.cfg")
            .takes_value(true))
        .get_matches();

    // Generate owned app arguments
    AppArgs {
        config: String::from(matches.value_of("config").unwrap())
    }
}

fn main() {
    let _logger = ConfigLogger::init();

    info!("starting up");

    // Run control interface (in its own thread)
    ControlListenBuilder::new().run();

    // Run server (in main thread)
    ServerListenBuilder::new().run();

    error!("could not start");
}
