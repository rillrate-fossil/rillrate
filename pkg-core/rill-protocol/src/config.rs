use anyhow::Error;
use once_cell::sync::OnceCell;
use std::convert::identity;
use std::env;
use std::fmt;
use std::str::FromStr;

/// Configuration parameter that can be overriden by an environment variable
/// or explicitly by setting default value.
#[derive(Debug)]
pub struct ConfigPatch<T> {
    /// High-priority value
    pre: &'static str,
    /// Low-priority value
    post: OnceCell<T>,
}

impl<T> ConfigPatch<T> {
    pub const fn new(pre: &'static str) -> Self {
        Self {
            pre,
            post: OnceCell::new(),
        }
    }

    pub const fn var(&self) -> &'static str {
        self.pre
    }

    pub fn env_var(&self) -> Result<Option<T>, Error>
    where
        T: FromStr,
    {
        if let Ok(s) = env::var(self.pre) {
            let value = s.parse().map_err(|_err| {
                Error::msg(format!("Can't parse {} variable from '{}'.", self.pre, s))
            })?;
            Ok(Some(value))
        } else {
            Ok(None)
        }
    }

    /// Offers an alternative default if no other value provided.
    pub fn offer(&self, value: T) {
        if let Err(_err) = self.post.set(value) {
            log::error!("Config value {} already overriden.", self.pre);
        }
    }

    pub fn get<F, D>(&self, value: F, default: D) -> T
    where
        T: fmt::Debug + FromStr + Clone,
        F: Fn() -> Option<T>,
        D: Fn() -> T,
    {
        let value = self
            .env_var()
            .map_err(|err| {
                log::error!("Default value for {} will be used: {}", self.pre, err);
            })
            .ok()
            .and_then(identity)
            .or_else(value)
            .or_else(|| self.post.get().cloned())
            .unwrap_or_else(default);
        log::debug!("{} = {:?}", self.pre, value);
        value
    }
}
