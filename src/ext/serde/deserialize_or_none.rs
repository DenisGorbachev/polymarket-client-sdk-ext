use serde::de::Error as DeError;
use serde::{Deserialize, Deserializer};
use std::fmt::Display;
use std::marker::PhantomData;
use std::str::FromStr;

pub struct DeserializeOrNone<T>(PhantomData<T>);

impl<T> DeserializeOrNone<T>
where
    T: FromStr,
    <T as FromStr>::Err: Display,
{
    pub fn run<'de, D>(deserializer: D) -> Result<Option<T>, D::Error>
    where
        D: Deserializer<'de>,
    {
        let s: String = Deserialize::deserialize(deserializer)?;
        if s.is_empty() { Ok(None) } else { T::from_str(&s).map(Some).map_err(DeError::custom) }
    }
}
