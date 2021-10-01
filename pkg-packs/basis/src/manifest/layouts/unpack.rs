use serde::de::{Deserialize, Deserializer, Error};
use std::fmt::Display;
use std::str::FromStr;

pub fn from_str<'de, T, D>(deserializer: D) -> Result<T, D::Error>
where
    T: FromStr,
    T::Err: Display,
    D: Deserializer<'de>,
{
    let s = <String>::deserialize(deserializer)?;
    T::from_str(&s).map_err(Error::custom)
}
