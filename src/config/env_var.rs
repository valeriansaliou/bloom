// Bloom
//
// HTTP REST API caching middleware
// Copyright: 2017, Valerian Saliou <valerian@valeriansaliou.name>
// License: Mozilla Public License v2.0 (MPL v2.0)

use regex::Regex;
use serde::{Deserialize, Deserializer};
use std::net::SocketAddr;
use std::path::PathBuf;

#[derive(Deserialize, PartialEq)]
struct WrappedString(String);

pub fn str<'de, D>(deserializer: D) -> Result<String, D::Error>
where
    D: Deserializer<'de>,
{
    let value = String::deserialize(deserializer)?;

    match is_env_var(&value) {
        true => Ok(get_env_var_str(&value)),
        false => Ok(value),
    }
}

pub fn opt_str<'de, D>(deserializer: D) -> Result<Option<String>, D::Error>
where
    D: Deserializer<'de>,
{
    Option::<WrappedString>::deserialize(deserializer).map(|option: Option<WrappedString>| {
        option.map(|wrapped: WrappedString| {
            let value = wrapped.0;

            match is_env_var(&value) {
                true => get_env_var_str(&value),
                false => value,
            }
        })
    })
}

pub fn socket_addr<'de, D>(deserializer: D) -> Result<SocketAddr, D::Error>
where
    D: Deserializer<'de>,
{
    let value = String::deserialize(deserializer)?;

    match is_env_var(&value) {
        true => Ok(get_env_var_str(&value).parse().unwrap()),
        false => Ok(value.parse().unwrap()),
    }
}

pub fn path_buf<'de, D>(deserializer: D) -> Result<PathBuf, D::Error>
where
    D: Deserializer<'de>,
{
    let value = String::deserialize(deserializer)?;

    match is_env_var(&value) {
        true => Ok(PathBuf::from(get_env_var_str(&value))),
        false => Ok(PathBuf::from(value)),
    }
}

pub fn bool<'de,D>(deserializer: D) -> Result<bool, D::Error>
where
    D: Deserializer<'de>,
{
    let value = String::deserialize(deserializer)?;

    match is_env_var(&value) {
        true => Ok(get_env_var_bool(&value)),
        false => Ok(value.parse().unwrap()),
    }
}

fn is_env_var(value: &str) -> bool {
    Regex::new(r"^\$\{[A-Z_0-9]+\}$")
        .expect("env_var: regex is invalid")
        .is_match(value)
}

fn get_env_var_str(wrapped_key: &str) -> String {
    let key: String = String::from(wrapped_key)
        .drain(2..(wrapped_key.len() - 1))
        .collect();

    std::env::var(key.clone()).unwrap_or_else(|_| panic!("env_var: variable '{}' is not set", key))
}

fn get_env_var_bool(wrapped_key: &str) -> bool {
    let key: String = String::from(wrapped_key)
        .drain(2..(wrapped_key.len() - 1))
        .collect();

    let value = std::env::var(key.clone())
        .unwrap_or_else(|_| panic!("env_var: variable '{}' is not set", key))
        .to_lowercase();
    match value.as_ref() {
        "0" | "false" => false,
        "1" | "true" => true,
        _ => panic!("env_var: variable '{}' is not a boolean", key),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn it_checks_environment_variable_patterns() {
        assert!(is_env_var("${XXX}"));
        assert!(is_env_var("${XXX_123}"));
        assert!(is_env_var("${123}"));
        assert!(is_env_var("${1_23}"));
        assert!(!is_env_var("${XXX"));
        assert!(!is_env_var("${XXX}a"));
        assert!(!is_env_var("a${XXX}"));
        assert!(!is_env_var("{XXX}"));
        assert!(!is_env_var("$XXX}"));
        assert!(!is_env_var("${envXXX}"));
        assert!(!is_env_var("${.XXX}"));
        assert!(!is_env_var("${XXX.}"));
        assert!(!is_env_var("${éíá}"));
        assert!(!is_env_var("${ÉÍÁ}"));
    }

    #[test]
    fn it_gets_string_value() {
        std::env::set_var("TEST", "test");

        assert_eq!(get_env_var_str("${TEST}"), "test");

        std::env::remove_var("TEST");
    }

    #[test]
    fn it_gets_bool_value() {
        std::env::set_var("TEST_BOOL_STR_TRUE", "true");
        std::env::set_var("TEST_BOOL_STR_FALSE", "false");
        std::env::set_var("TEST_BOOL_STR_TRUE_UP", "TRUE");
        std::env::set_var("TEST_BOOL_STR_FALSE_UP", "FALSE");
        std::env::set_var("TEST_BOOL_STR_1", "1");
        std::env::set_var("TEST_BOOL_STR_0", "0");

        assert_eq!(get_env_var_bool("${TEST_BOOL_STR_TRUE}"), true);
        assert_eq!(get_env_var_bool("${TEST_BOOL_STR_FALSE}"), false);
        assert_eq!(get_env_var_bool("${TEST_BOOL_STR_TRUE_UP}"), true);
        assert_eq!(get_env_var_bool("${TEST_BOOL_STR_FALSE_UP}"), false);
        assert_eq!(get_env_var_bool("${TEST_BOOL_STR_1}"), true);
        assert_eq!(get_env_var_bool("${TEST_BOOL_STR_0}"), false);

        std::env::remove_var("TEST_BOOL_STR_TRUE");
        std::env::remove_var("TEST_BOOL_STR_FALSE");
        std::env::remove_var("TEST_BOOL_STR_1");
        std::env::remove_var("TEST_BOOL_STR_0");
    }
}
