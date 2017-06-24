// Bloom
//
// HTTP REST API caching middleware
// Copyright: 2017, Valerian Saliou <valerian@valeriansaliou.name>
// License: Mozilla Public License v2.0 (MPL v2.0)

#[macro_use]
extern crate log;

#[macro_use]
extern crate clap;

mod config;
mod server;
mod proxy;
mod cache;

use clap::{App, Arg};
use config::logger::Logger;
use config::reader::ReaderBuilder;
use proxy::serve::ServeBuilder;
use server::listen::ListenBuilder;

fn main() {
    let _logger = Logger::init();

    info!("starting up");

    let app = App::new(crate_name!())
                .version(crate_version!())
                .author(crate_authors!("\n"))
                .about(crate_description!())
                .arg(Arg::with_name("config")
                    .short("c")
                    .long("config")
                    .help("Path to configuration file")
                    .default_value("./config.cfg")
                    .takes_value(true));

    let args = app.get_matches();
    let conf = ReaderBuilder::new().read(args.value_of("config").unwrap());

    // Connect to cache backend
    // TODO

    // Create serve manager
    let serve = ServeBuilder::new(conf.proxy);

    // Run server (in main thread)
    ListenBuilder::new(conf.listen).run(serve);

    error!("could not start");
}
