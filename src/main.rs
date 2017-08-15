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
extern crate ini;
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
use std::time::Duration;

use clap::{App, Arg};
use futures::Future;

use config::config::Config;
use config::logger::ConfigLogger;
use config::reader::ConfigReader;
use cache::store::{CacheStore, CacheStoreBuilder};
use proxy::serve::{ProxyServe, ProxyServeBuilder};
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
    static ref APP_PROXY_SERVE: ProxyServe = ProxyServeBuilder::new();
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
    // TODO: ensure args
    // TODO: ensure conf

    if APP_CACHE_STORE.ensure().wait().is_err() {
        panic!("could not ensure cache store");
    }
    if APP_PROXY_SERVE.ensure().is_err() {
        panic!("could not ensure proxy serve");
    }
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
    let _logger = ConfigLogger::init();

    info!("starting up");

    // Ensure all states are bound
    ensure_states();

    // Run control interface (in its own thread)
    ControlListenBuilder::new().run();

    // Run server (from main thread, maintain thread active if down)
    spawn_worker();

    error!("could not start");
}
