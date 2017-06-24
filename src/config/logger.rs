// Bloom
//
// HTTP REST API caching middleware
// Copyright: 2017, Valerian Saliou <valerian@valeriansaliou.name>
// License: Mozilla Public License v2.0 (MPL v2.0)

extern crate log;

use log::{LogRecord, LogLevel, LogMetadata, LogLevelFilter, SetLoggerError};

pub struct Logger;

impl log::Log for Logger {
    fn enabled(&self, metadata: &LogMetadata) -> bool {
        metadata.level() <= LogLevel::Debug
    }

    fn log(&self, record: &LogRecord) {
        if self.enabled(record.metadata()) {
            println!("({}) - {}", record.level(), record.args());
        }
    }
}

impl Logger {
    pub fn init() -> Result<(), SetLoggerError> {
        log::set_logger(|max_log_level| {
            max_log_level.set(LogLevelFilter::Debug);
            Box::new(Logger)
        })
    }
}
