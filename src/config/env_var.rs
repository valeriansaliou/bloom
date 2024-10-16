// Bloom
//
// HTTP REST API caching middleware
// Copyright: 2017, Valerian Saliou <valerian@valeriansaliou.name>
// License: Mozilla Public License v2.0 (MPL v2.0)

use regex::Regex;
use serde::{de, Deserialize, Deserializer};
use std::net::SocketAddr;
use toml::Value;

#[derive(Deserialize, PartialEq)]
struct WrappedString(String);

pub fn str<'de, D>(deserializer: D) -> Result<String, D::Error>
where
    D: Deserializer<'de>,
{
    let value = String::deserialize(deserializer)?;

    if is_env_var(&value) {
        Ok(get_env_var_str(&value))
    } else {
        Ok(value)
    }
}

pub fn opt_str<'de, D>(deserializer: D) -> Result<Option<String>, D::Error>
where
    D: Deserializer<'de>,
{
    Option::<WrappedString>::deserialize(deserializer).map(|option: Option<WrappedString>| {
        option.map(|wrapped: WrappedString| {
            let value = wrapped.0;

            if is_env_var(&value) {
                get_env_var_str(&value)
            } else {
                value
            }
        })
    })
}

pub fn socket_addr<'de, D>(deserializer: D) -> Result<SocketAddr, D::Error>
where
    D: Deserializer<'de>,
{
    let value = String::deserialize(deserializer)?;

    if is_env_var(&value) {
        Ok(get_env_var_str(&value).parse().unwrap())
    } else {
        Ok(value.parse().unwrap())
    }
}

pub fn bool<'de, D>(deserializer: D) -> Result<bool, D::Error>
where
    D: Deserializer<'de>,
{
    Ok(match Value::deserialize(deserializer)? {
        Value::Boolean(b) => b,
        Value::String(s) => {
            if is_env_var(&s) {
                get_env_var_bool(&s)
            } else {
                s.parse().unwrap()
            }
        }
        _ => return Err(de::Error::custom("Wrong type: expected boolean or string")),
    })
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

    std::env::var(key.clone()).unwrap_or_else(|_| panic!("env_var: variable '{key}' is not set"))
}

fn get_env_var_bool(wrapped_key: &str) -> bool {
    let key: String = String::from(wrapped_key)
        .drain(2..(wrapped_key.len() - 1))
        .collect();

    let value = std::env::var(key.clone())
        .unwrap_or_else(|_| panic!("env_var: variable '{key}' is not set"))
        .to_lowercase();
    match value.as_ref() {
        "0" | "false" => false,
        "1" | "true" => true,
        _ => panic!("env_var: variable '{key}' is not a boolean"),
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

        assert!(get_env_var_bool("${TEST_BOOL_STR_TRUE}"));
        assert!(!get_env_var_bool("${TEST_BOOL_STR_FALSE}"));
        assert!(get_env_var_bool("${TEST_BOOL_STR_TRUE_UP}"));
        assert!(!get_env_var_bool("${TEST_BOOL_STR_FALSE_UP}"));
        assert!(get_env_var_bool("${TEST_BOOL_STR_1}"));
        assert!(!get_env_var_bool("${TEST_BOOL_STR_0}"));

        std::env::remove_var("TEST_BOOL_STR_TRUE");
        std::env::remove_var("TEST_BOOL_STR_FALSE");
        std::env::remove_var("TEST_BOOL_STR_TRUE_UP");
        std::env::remove_var("TEST_BOOL_STR_FALSE_UP");
        std::env::remove_var("TEST_BOOL_STR_1");
        std::env::remove_var("TEST_BOOL_STR_0");
    }
}
