// Bloom
//
// HTTP REST API caching middleware
// Copyright: 2017, Valerian Saliou <valerian@valeriansaliou.name>
// License: Mozilla Public License v2.0 (MPL v2.0)

use std::str;
use std::io::{Read, Write, ErrorKind};
use std::result::Result;
use std::net::TcpStream;
use std::time::Duration;
use rand::{thread_rng, Rng};
use rand::distributions::Alphanumeric;

use super::command::ControlCommandResponse;
use super::command::ControlCommand;
use super::command::COMMAND_SIZE;
use crate::cache::route::CacheRoute;
use crate::cache::route::ROUTE_HASH_SIZE;
use crate::APP_CONF;
use crate::LINE_FEED;

pub struct ControlHandle;

enum ControlHandleError {
    Closed,
    IncompatibleHasher,
    NotRecognized,
    TimedOut,
    ConnectionAborted,
    Interrupted,
    Unknown,
}

#[derive(PartialEq)]
enum ControlHandleMessageResult {
    Continue,
    Close,
}

const LINE_END_GAP: usize = 1;
const MAX_LINE_SIZE: usize = COMMAND_SIZE + ROUTE_HASH_SIZE + LINE_END_GAP + 1;
const HASH_VALUE_SIZE: usize = 10;
const HASH_RESULT_SIZE: usize = 7 + ROUTE_HASH_SIZE + LINE_END_GAP + 1;
const SHARD_DEFAULT: ControlShard = 0;
const TCP_TIMEOUT_NON_ESTABLISHED: u64 = 20;

static BUFFER_LINE_SEPARATOR: u8 = '\n' as u8;

pub type ControlShard = u8;

lazy_static! {
    static ref CONNECTED_BANNER: String = format!("CONNECTED <{} v{}>",
        env!("CARGO_PKG_NAME"), env!("CARGO_PKG_VERSION"));
}

impl ControlHandleError {
    pub fn to_str(&self) -> &'static str {
        match *self {
            ControlHandleError::Closed => "closed",
            ControlHandleError::IncompatibleHasher => "incompatible_hasher",
            ControlHandleError::NotRecognized => "not_recognized",
            ControlHandleError::TimedOut => "timed_out",
            ControlHandleError::ConnectionAborted => "connection_aborted",
            ControlHandleError::Interrupted => "interrupted",
            ControlHandleError::Unknown => "unknown",
        }
    }
}

impl ControlHandle {
    pub fn client(mut stream: TcpStream) {
        // Configure stream (non-established)
        ControlHandle::configure_stream(&stream, false);

        // Send connected banner
        write!(stream, "{}{}", *CONNECTED_BANNER, LINE_FEED).expect("write failed");

        // Ensure client hasher is compatible
        match Self::ensure_hasher(&stream) {
            Ok(_) => {
                // Configure stream (established)
                ControlHandle::configure_stream(&stream, true);

                // Send started acknowledgement
                write!(stream, "STARTED{}", LINE_FEED).expect("write failed");

                // Select default shard
                let mut shard = SHARD_DEFAULT;

                // Initialize packet buffer
                let mut buffer = Vec::new();

                // Wait for incoming messages
                'handler: loop {
                    let mut read = [0; MAX_LINE_SIZE];

                    match stream.read(&mut read) {
                        Ok(n) => {
                            // Should close?
                            if n == 0 {
                                break;
                            }

                            // Buffer chunk
                            buffer.extend_from_slice(&read[0..n]);

                            // Should handle this chunk? (terminated)
                            if buffer[buffer.len() - 1] == BUFFER_LINE_SEPARATOR {
                                {
                                    // Handle all buffered chunks as lines
                                    let buffer_split =
                                        buffer.split(|value| value == &BUFFER_LINE_SEPARATOR);

                                    for line in buffer_split {
                                        if line.is_empty() == false {
                                            if Self::on_message(&mut shard, &stream, line) ==
                                                ControlHandleMessageResult::Close
                                            {
                                                // Should close?
                                                break 'handler;
                                            }
                                        }
                                    }
                                }

                                // Reset buffer
                                buffer.clear();
                            }
                        }
                        Err(err) => {
                            info!("closing control thread with traceback: {}", err);

                            panic!("closing control channel");
                        }
                    }
                }
            }
            Err(err) => {
                write!(stream, "ENDED {}{}", err.to_str(), LINE_FEED).expect("write failed");
            }
        }
    }

    fn configure_stream(stream: &TcpStream, is_established: bool) {
        let tcp_timeout = if is_established == true {
            APP_CONF.control.tcp_timeout
        } else {
            TCP_TIMEOUT_NON_ESTABLISHED
        };

        assert!(stream.set_nodelay(true).is_ok());

        assert!(
            stream
                .set_read_timeout(Some(Duration::new(tcp_timeout, 0)))
                .is_ok()
        );
        assert!(
            stream
                .set_write_timeout(Some(Duration::new(tcp_timeout, 0)))
                .is_ok()
        );
    }

    fn ensure_hasher(mut stream: &TcpStream) -> Result<Option<()>, ControlHandleError> {
        let test_value: String = thread_rng()
            .sample_iter(&Alphanumeric)
            .take(HASH_VALUE_SIZE)
            .collect();
        let test_hash = CacheRoute::hash(test_value.as_str());

        write!(stream, "HASHREQ {}{}", test_value, LINE_FEED).expect("write failed");

        debug!(
            "sent hasher request: {} and expecting hash: {}",
            test_value,
            test_hash
        );

        loop {
            let mut read = [0; HASH_RESULT_SIZE];

            match stream.read(&mut read) {
                Ok(n) => {
                    if n == 0 {
                        return Err(ControlHandleError::Closed);
                    }

                    let mut parts = str::from_utf8(&read[0..n]).unwrap_or("").split_whitespace();

                    if parts.next().unwrap_or("") == "HASHRES" {
                        let res_hash = parts.next().unwrap_or("");

                        debug!(
                            "got hasher response: {} and expecting: {}",
                            res_hash,
                            test_hash
                        );

                        // Validate hash
                        if res_hash.is_empty() == false && res_hash == test_hash {
                            return Ok(None);
                        }

                        return Err(ControlHandleError::IncompatibleHasher);
                    }

                    return Err(ControlHandleError::NotRecognized);
                }
                Err(err) => {
                    let err_reason = match err.kind() {
                        ErrorKind::TimedOut => ControlHandleError::TimedOut,
                        ErrorKind::ConnectionAborted => ControlHandleError::ConnectionAborted,
                        ErrorKind::Interrupted => ControlHandleError::Interrupted,
                        _ => ControlHandleError::Unknown,
                    };

                    return Err(err_reason);
                }
            }
        }
    }

    fn on_message(
        shard: &mut ControlShard,
        mut stream: &TcpStream,
        message_slice: &[u8],
    ) -> ControlHandleMessageResult {
        let message = str::from_utf8(message_slice).unwrap_or("");

        debug!("got control message on shard {}: {}", shard, message);

        let mut result = ControlHandleMessageResult::Continue;

        let response = match Self::handle_message(shard, &message) {
            Ok(resp) => {
                match resp {
                    ControlCommandResponse::Ok |
                    ControlCommandResponse::Pong |
                    ControlCommandResponse::Ended |
                    ControlCommandResponse::Nil |
                    ControlCommandResponse::Void => {
                        if resp == ControlCommandResponse::Ended {
                            result = ControlHandleMessageResult::Close;
                        }
                        resp.to_str()
                    }
                    _ => ControlCommandResponse::Err.to_str(),
                }
            }
            _ => ControlCommandResponse::Err.to_str(),
        };

        if response.is_empty() == false {
            write!(stream, "{}{}", response, LINE_FEED).expect("write failed");

            debug!("wrote response: {}", response);
        }

        return result;
    }

    fn handle_message(
        shard: &mut ControlShard,
        message: &str,
    ) -> Result<ControlCommandResponse, Option<()>> {
        let mut parts = message.split_whitespace();
        let command = parts.next().unwrap_or("");

        debug!("will dispatch command: {}", command);

        match command {
            "" => Ok(ControlCommandResponse::Void),
            "FLUSHB" => ControlCommand::dispatch_flush_bucket(shard, parts),
            "FLUSHA" => ControlCommand::dispatch_flush_auth(shard, parts),
            "PING" => ControlCommand::dispatch_ping(),
            "SHARD" => ControlCommand::dispatch_shard(shard, parts),
            "QUIT" => ControlCommand::dispatch_quit(),
            _ => Ok(ControlCommandResponse::Nil),
        }
    }
}
