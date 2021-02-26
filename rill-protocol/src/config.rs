use anyhow::Error;
use once_cell::sync::OnceCell;
use std::env;
use std::str::FromStr;

pub struct ConfigPatch<T> {
    pre: &'static str,
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

    pub fn env_var(&self) -> Result<T, Error>
    where
        T: FromStr,
    {
        let s = env::var(self.pre)?;
        let value = s.parse().map_err(|_err| {
            Error::msg(format!("Can't parse {} variable from '{}'.", self.pre, s))
        })?;
        Ok(value)
    }

    pub fn set(&self, value: T) {
        if let Err(_err) = self.post.set(value) {
            log::error!("Config value {} already overriden.", self.pre);
        }
    }

    pub fn get<F, D>(&self, value: F, default: D) -> T
    where
        T: FromStr + Clone,
        F: Fn() -> Option<T>,
        D: Fn() -> T,
    {
        self
            .env_var()
            .map_err(|err| {
                log::error!("Default value for {} will be used: {}", self.pre, err);
            })
            .ok()
            .or_else(value)
            .or_else(|| self.post.get().cloned())
            .unwrap_or_else(default)
    }
}
