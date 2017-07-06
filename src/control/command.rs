// Bloom
//
// HTTP REST API caching middleware
// Copyright: 2017, Valerian Saliou <valerian@valeriansaliou.name>
// License: Mozilla Public License v2.0 (MPL v2.0)

use std::str::SplitWhitespace;

#[derive(PartialEq)]
pub enum ControlCommandResponse {
    Nil,
    Ok,
    Pong,
    Ended,
    Err
}

impl ControlCommandResponse {
    pub fn to_str(&self) -> &'static str {
        match *self {
            ControlCommandResponse::Nil => "NIL",
            ControlCommandResponse::Ok => "OK",
            ControlCommandResponse::Pong => "PONG",
            ControlCommandResponse::Ended => "ENDED quit",
            ControlCommandResponse::Err => "ERR"
        }
    }
}

pub struct ControlCommand;

pub const COMMAND_SIZE: usize = 6;

impl ControlCommand {
    pub fn dispatch_flush_bucket(mut parts: SplitWhitespace) ->
        Result<ControlCommandResponse, u8> {
        let namespace = parts.next().unwrap_or("");

        debug!("dispatch bucket flush for namespace: {}", namespace);

        if namespace.is_empty() == false {
            // TODO

            // let ns = CacheRoute::gen_ns(shard, req.version(), req.method(),
            //   req.path(), req.query(), auth);

            // CacheStore::purge(ns);

            // return Ok(ControlCommandResponse::Ok)
        }

        Err(0)
    }

    pub fn dispatch_flush_auth(mut parts: SplitWhitespace) ->
        Result<ControlCommandResponse, u8> {
        let auth = parts.next().unwrap_or("");

        debug!("dispatch auth flush for auth: {}", auth);

        if auth.is_empty() == false {
            // TODO

            // let ns = CacheRoute::gen_ns(shard, req.version(), req.method(),
            //   req.path(), req.query(), auth);

            // CacheStore::purge(ns);

            // return Ok(ControlCommandResponse::Ok)
        }

        Err(0)
    }

    pub fn dispatch_ping() -> Result<ControlCommandResponse, u8> {
        Ok(ControlCommandResponse::Pong)
    }

    pub fn dispatch_quit() -> Result<ControlCommandResponse, u8> {
        Ok(ControlCommandResponse::Ended)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn it_matches_command_response_string() {
        assert_eq!(ControlCommandResponse::Ok.to_str(), "OK");
        assert_eq!(ControlCommandResponse::Pong.to_str(), "PONG");
        assert_eq!(ControlCommandResponse::Ended.to_str(), "ENDED");
        assert_eq!(ControlCommandResponse::Err.to_str(), "ERR");
    }
}
