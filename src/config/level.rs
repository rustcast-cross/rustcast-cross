//! Parser for log levels

use serde::{Deserialize, Deserializer, Serialize, Serializer, de::Visitor};
use std::str::FromStr;
use tracing::Level;

pub fn deserialize<'de, D>(deserializer: D) -> Result<Level, D::Error>
where
    D: Deserializer<'de>,
{
    struct LevelVisitor;

    impl Visitor<'_> for LevelVisitor {
        type Value = Level;

        fn visit_str<E>(self, v: &str) -> Result<Self::Value, E>
        where
            E: serde::de::Error,
        {
            match v.to_uppercase().as_str() {
                "ERROR" => Ok(Level::ERROR),
                "WARN" => Ok(Level::WARN),
                "INFO" => Ok(Level::INFO),
                "DEBUG" => Ok(Level::DEBUG),
                "TRACE" => Ok(Level::TRACE),
                v => Err(E::custom(format!(
                    "Invalid log level {v}; must be one of \"ERROR\", \"WARN\", \"INFO\", \"DEBUG\"\
                    , or \"TRACE\" (case insensitive)"
                ))),
            }
        }

        fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
            write!(
                formatter,
                "a string with the text \"ERROR\", \"WARN\", \"INFO\", \"DEBUG\" or \
                \"TRACE\" (case insensitive)"
            )
        }
    }

    deserializer.deserialize_str(LevelVisitor)
}

#[allow(clippy::trivially_copy_pass_by_ref)] // because serde wants it like this
pub fn serialize<S>(level: &Level, serializer: S) -> Result<S::Ok, S::Error>
where
    S: Serializer,
{
    serializer.serialize_str(match *level {
        Level::ERROR => "ERROR",
        Level::WARN => "WARN",
        Level::INFO => "INFO",
        Level::DEBUG => "DEBUG",
        Level::TRACE => "TRACE",
    })
}
