use crate::ReadableConfig;
use anyhow::Error;
use std::env::{var, VarError};
use std::str::FromStr;

pub fn typed_var<T>(name: &'static str) -> Result<Option<T>, Error>
where
    T: FromStr,
    Error: From<T::Err>,
{
    match var(name) {
        Ok(value) => {
            let t = value.parse().map_err(Error::from)?;
            Ok(Some(t))
        }
        Err(VarError::NotPresent) => Ok(None),
        Err(err) => Err(err.into()),
    }
}

pub trait FromEnv: ReadableConfig {
    fn prefix() -> &'static str;

    fn from_env() -> Result<Self, Error> {
        envy::prefixed(Self::prefix())
            .from_env()
            .map_err(Error::from)
    }
}
