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
extern crate brotli;
extern crate futures;
extern crate futures_cpupool;
extern crate rand;
extern crate r2d2;
extern crate r2d2_redis;
extern crate redis;
extern crate unicase;

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
use log::LevelFilter;

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

pub static THREAD_NAME_WORKER: &'static str = "bloom-worker";
pub static THREAD_NAME_CONTROL_MASTER: &'static str = "bloom-control-master";
pub static THREAD_NAME_CONTROL_CLIENT: &'static str = "bloom-control-client";

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
    AppArgs { config: String::from(matches.value_of("config").expect("invalid config value")) }
}

fn ensure_states() {
    // Ensure all statics are valid (a `deref` is enough to lazily initialize them)
    let (_, _, _) = (APP_ARGS.deref(), APP_CONF.deref(), APP_CACHE_STORE.deref());
}

fn spawn_worker() {
    let worker = thread::Builder::new()
        .name(THREAD_NAME_WORKER.to_string())
        .spawn(|| ServerListenBuilder::new().run());

    // Block on worker thread (join it)
    let has_error = if let Ok(worker_thread) = worker {
        worker_thread.join().is_err()
    } else {
        true
    };

    // Worker thread crashed?
    if has_error == true {
        error!("worker thread crashed, setting it up again");

        // Prevents thread start loop floods
        thread::sleep(Duration::from_secs(1));

        spawn_worker();
    }
}

fn main() {
    let _logger = ConfigLogger::init(LevelFilter::from_str(&APP_CONF.server.log_level).expect(
        "invalid log level",
    ));

    info!("starting up");

    // Ensure all states are bound
    ensure_states();

    // Run control interface (in its own thread)
    ControlListenBuilder::new().run();

    // Run server (from main thread, maintain thread active if down)
    spawn_worker();

    error!("could not start");
}
