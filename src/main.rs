// Bloom
//
// HTTP REST API caching middleware
// Copyright: 2017, Valerian Saliou <valerian@valeriansaliou.name>
// License: Mozilla Public License v2.0 (MPL v2.0)

#[macro_use]
extern crate log;

mod config;
mod server;
mod proxy;
mod cache;

use config::logger::Logger;
use config::reader::ReaderBuilder;
use server::listen::ServerListenBuilder;

static MODULE: &'static str = "main";

fn main() {
    let _logger = Logger::init();

    info!("[{}] starting up", MODULE);

    let conf = ReaderBuilder::new().read("config.cfg");

    // Connect to cache backend
    // TODO

    // Run server (in main thread)
    ServerListenBuilder::new(conf.listen).run();

    error!("[{}] could not start", MODULE);
}
