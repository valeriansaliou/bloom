// Bloom
//
// HTTP REST API caching middleware
// Copyright: 2017, Valerian Saliou <valerian@valeriansaliou.name>
// License: Mozilla Public License v2.0 (MPL v2.0)

#[macro_use]
extern crate log;
#[macro_use]
extern crate clap;
#[macro_use]
extern crate lazy_static;
#[macro_use]
extern crate serde_derive;
extern crate toml;
extern crate httparse;
extern crate hyper;
extern crate tokio_core;
extern crate farmhash;
extern crate futures;
extern crate rand;
extern crate r2d2;
extern crate r2d2_redis;
extern crate redis;

mod config;
mod header;
mod proxy;
mod cache;
mod control;
mod server;

use std::thread;
use std::ops::Deref;
use std::time::Duration;
use std::str::FromStr;

use clap::{App, Arg};
use log::LogLevelFilter;

use config::config::Config;
use config::logger::ConfigLogger;
use config::reader::ConfigReader;
use cache::store::{CacheStore, CacheStoreBuilder};
use control::listen::ControlListenBuilder;
use server::listen::ServerListenBuilder;

struct AppArgs {
    config: String,
}

pub static LINE_FEED: &'static str = "\r\n";

lazy_static! {
    static ref APP_ARGS: AppArgs = make_app_args();
    static ref APP_CONF: Config = ConfigReader::make();
    static ref APP_CACHE_STORE: CacheStore = CacheStoreBuilder::new();
}

fn make_app_args() -> AppArgs {
    let matches = App::new(crate_name!())
        .version(crate_version!())
        .author(crate_authors!("\n"))
        .about(crate_description!())
        .arg(
            Arg::with_name("config")
                .short("c")
                .long("config")
                .help("Path to configuration file")
                .default_value("./config.cfg")
                .takes_value(true),
        )
        .get_matches();

    // Generate owned app arguments
    AppArgs { config: String::from(matches.value_of("config").unwrap()) }
}

fn ensure_states() {
    // Ensure all statics are valid (a `deref` is enough to lazily initialize them)
    APP_ARGS.deref();
    APP_CONF.deref();
    APP_CACHE_STORE.deref();
}

fn spawn_worker() {
    let worker = thread::spawn(|| { ServerListenBuilder::new().run(); });

    if worker.join().is_err() == true {
        error!("worker thread crashed, setting it up again");

        // Prevents thread start loop floods
        thread::sleep(Duration::from_secs(1));

        spawn_worker();
    }
}

fn main() {
    let _logger = ConfigLogger::init(
        LogLevelFilter::from_str(&APP_CONF.server.log_level).expect("invalid log level"),
    );

    info!("starting up");

    // Ensure all states are bound
    ensure_states();

    // Run control interface (in its own thread)
    ControlListenBuilder::new().run();

    // Run server (from main thread, maintain thread active if down)
    spawn_worker();

    error!("could not start");
}
