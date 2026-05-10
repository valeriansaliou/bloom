// Bloom
//
// HTTP REST API caching middleware
// Copyright: 2026, Valerian Saliou <valerian@valeriansaliou.name>
// License: Mozilla Public License v2.0 (MPL v2.0)

use std::fs::OpenOptions;
use std::io::Write;
use std::path::PathBuf;
use std::sync::mpsc::{self, Receiver, Sender};
use std::thread;

use hyper::header::HeaderMap;
use time_format;

use crate::{APP_CONF, THREAD_NAME_PROXY_LOGGER};

pub struct ProxyLoggerBuilder;

pub struct ProxyLoggerRequest {
    pub method: String,
    pub uri: String,
    pub shard: u8,
    pub headers: HeaderMap,
}

pub type ProxyLogger = Sender<ProxyLoggerRequest>;

impl ProxyLoggerBuilder {
    pub fn new() -> Option<ProxyLogger> {
        // Acquire log file path (it will immediately skip and return if the request log is not \
        //   enabled — so that we do not start the tread needlessly)
        let log_path = APP_CONF.proxy.request_log.as_ref()?;

        info!("starting proxy logger thread");
        warn!("⚠️ writing all proxied requests to file: {:?}", log_path);

        // Create multi-producer, single-consumer FIFO queue
        let (sender, receiver) = mpsc::channel::<ProxyLoggerRequest>();

        // Spawn receiver thread
        thread::Builder::new()
            .name(THREAD_NAME_PROXY_LOGGER.to_string())
            .spawn(move || Self::run(log_path, receiver))
            .expect("could not spawn proxy logger thread");

        Some(sender)
    }

    fn run(log_path: &PathBuf, receiver: Receiver<ProxyLoggerRequest>) {
        // Open request log file
        // Notice: if the log file cannot be opened, then the thread will panic once, but Bloom \
        //   will still run just fine proxying requests.
        let mut log_file = OpenOptions::new()
            .create(true)
            .append(true)
            .open(log_path)
            .expect(&format!(
                "could not open proxy request log file: {:?}",
                log_path
            ));

        // Receive proxy requests to write to log
        while let Ok(entry) = receiver.recv() {
            let now_time = time_format::now().unwrap_or_default();

            // Format request metadata
            let mut line = format!(
                "\n[{}] [SHARD{}] {} {}\n\n",
                time_format::format_iso8601_utc(now_time).unwrap(),
                entry.shard,
                entry.method,
                entry.uri
            );

            // Append all request headers
            for (name, value) in &entry.headers {
                line.push_str(name.as_str());
                line.push_str(": ");
                line.push_str(value.to_str().unwrap_or("<binary>"));
                line.push('\n');
            }

            // Append request separator
            line.push_str("\n---\n");

            // Write request log to log file
            if let Err(err) = log_file.write_all(line.as_bytes()) {
                error!("could not write to proxy request log: {}", err);
            }
        }
    }
}
