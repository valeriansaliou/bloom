// Bloom
//
// HTTP REST API caching middleware
// Copyright: 2017, Valerian Saliou <valerian@valeriansaliou.name>
// License: Mozilla Public License v2.0 (MPL v2.0)

use std::str;
use std::io::{Read, Write};
use std::result::Result;
use std::net::{Shutdown, TcpStream};

use rand::{thread_rng, Rng};

use super::command::ControlCommandResponse;
use super::command::ControlCommand;
use super::command::COMMAND_SIZE;
use cache::route::CacheRoute;
use cache::route::ROUTE_SIZE;

pub struct ControlHandle;

const MAX_LINE_SIZE: usize = COMMAND_SIZE + ROUTE_SIZE + 1;
const HASH_VALUE_SIZE: usize = 20;
const HASH_RESULT_SIZE: usize = 24;

lazy_static! {
    static ref CONNECTED_BANNER: String = format!("CONNECTED <{} v{}>",
        env!("CARGO_PKG_NAME"), env!("CARGO_PKG_VERSION"));
}

impl ControlHandle {
    pub fn client(mut stream: TcpStream) {
        write!(stream, "{}\r\n", *CONNECTED_BANNER).expect("write failed");

        // Ensure client hasher is compatible
        match Self::test_hasher(&stream) {
            Ok(_) => {
                write!(stream, "STARTED\r\n").expect("write failed");

                // Wait for incoming messages
                loop {
                    let mut read = [0; MAX_LINE_SIZE];

                    match stream.read(&mut read) {
                        Ok(n) => {
                            if n == 0 ||
                                Self::on_message(&stream, &read[0..n]) == true {
                                // Should close?
                                break;
                            }
                        }
                        Err(_) => {
                            panic!("stream down");
                        }
                    }
                }
            },
            Err(err) => {
                write!(stream, "ENDED {}\r\n", err)
                    .expect("write failed");
            }
        }
    }

    pub fn test_hasher(mut stream: &TcpStream) ->
        Result<Option<bool>, &'static str> {
        let test_value: String = thread_rng().gen_ascii_chars()
                                    .take(HASH_VALUE_SIZE).collect();
        let test_hash = CacheRoute::hash(test_value.as_str());

        write!(stream, "HASHREQ {}\r\n", test_value).expect("write failed");

        debug!("sent hasher request: {} and expecting hash: {}",
            test_value, test_hash);

        loop {
            let mut read = [0; HASH_RESULT_SIZE];

            match stream.read(&mut read) {
                Ok(n) => {
                    if n == 0 {
                        return Err("closed")
                    }

                    let mut parts = str::from_utf8(&read[0..n])
                                        .unwrap_or("").split_whitespace();

                    if parts.next().unwrap_or("") == "HASHRES" {
                        let res_hash = parts.next().unwrap_or("");

                        debug!("got hasher response: {} and expecting: {}",
                            res_hash, test_hash);

                        // Validate hash
                        if res_hash.is_empty() == false &&
                            res_hash == test_hash {
                            return Ok(None)
                        }

                        return Err("incompatible_hasher")
                    }

                    return Err("not_recognized")
                }
                _ => {
                    return Err("unknown")
                }
            }
        }
    }

    pub fn on_message(mut stream: &TcpStream, message_slice: &[u8]) -> bool {
        let message = str::from_utf8(message_slice).unwrap_or("");

        debug!("got control message: {}", message);

        let mut do_shutdown = false;

        let response = match Self::handle_message(&message) {
            Ok(resp) => match resp {
                ControlCommandResponse::Ok
                | ControlCommandResponse::Pong
                | ControlCommandResponse::Ended
                | ControlCommandResponse::Nil
                | ControlCommandResponse::Void => {
                    if resp == ControlCommandResponse::Ended {
                        do_shutdown = true
                    }
                    resp.to_str()
                },
                _ => ControlCommandResponse::Err.to_str()
            },
            _ => ControlCommandResponse::Err.to_str()
        };

        if response.is_empty() == false {
            write!(stream, "{}\r\n", response).expect("write failed");
        }

        return do_shutdown
    }

    pub fn handle_message(message: &str) ->
        Result<ControlCommandResponse, Option<bool>> {
        let mut parts = message.split_whitespace();
        let command = parts.next().unwrap_or("");

        match command {
            "" => Ok(ControlCommandResponse::Void),
            "FLUSHB" => ControlCommand::dispatch_flush_bucket(parts),
            "FLUSHA" => ControlCommand::dispatch_flush_auth(parts),
            "PING" => ControlCommand::dispatch_ping(),
            "QUIT" => ControlCommand::dispatch_quit(),
            _ => Ok(ControlCommandResponse::Nil)
        }
    }
}
