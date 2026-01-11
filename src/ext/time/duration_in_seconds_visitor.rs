use serde::Deserializer;
use serde::de::{Error, Visitor};
use std::fmt;
use time::Duration;

pub struct DurationInSecondsVisitor;

impl Visitor<'_> for DurationInSecondsVisitor {
    type Value = Duration;

    fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        formatter.write_str("an integer representing the number of seconds")
    }

    fn visit_i64<E>(self, v: i64) -> Result<Self::Value, E>
    where
        E: Error,
    {
        Ok(Duration::seconds(v))
    }

    fn visit_u64<E>(self, v: u64) -> Result<Self::Value, E>
    where
        E: Error,
    {
        let v = i64::try_from(v).map_err(|e| E::custom(format!("Failed to parse u64 as i64: {}", e)))?;
        Ok(Duration::seconds(v))
    }
}

impl DurationInSecondsVisitor {
    pub fn deserialize<'de, D>(deserializer: D) -> Result<Duration, D::Error>
    where
        D: Deserializer<'de>,
    {
        deserializer.deserialize_i64(DurationInSecondsVisitor)
    }
}
