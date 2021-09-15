pub mod env;

use anyhow::Error;
use async_trait::async_trait;
pub use env::FromEnv;
use meio::LiteTask;
use serde::{de::DeserializeOwned, Serialize};
use std::fmt::Debug;
use std::marker::PhantomData;
use std::path::PathBuf;
use tokio::fs::{remove_file, File};
use tokio::io::{AsyncReadExt, AsyncWriteExt};

pub trait Config: Debug + Send + 'static {}

#[async_trait]
pub trait ReadableConfig: Config + DeserializeOwned {
    async fn read(path: PathBuf) -> Result<Self, Error> {
        let mut file = File::open(path).await?;
        let mut contents = Vec::new();
        file.read_to_end(&mut contents).await?;
        let config: Self = toml::from_slice(&contents)?;
        Ok(config)
    }
}

pub struct ReadConfigFile<T> {
    path: Option<PathBuf>,
    default_path: &'static str,
    _config: PhantomData<T>,
}

impl<T> ReadConfigFile<T> {
    pub fn new(path: Option<PathBuf>, default_path: &'static str) -> Self {
        Self {
            path,
            default_path,
            _config: PhantomData,
        }
    }
}

#[async_trait]
impl<T: ReadableConfig> LiteTask for ReadConfigFile<T> {
    type Output = Option<T>;

    async fn interruptable_routine(mut self) -> Result<Self::Output, Error> {
        let config = {
            if let Some(path) = self.path {
                Some(T::read(path).await?)
            } else {
                let path = self.default_path.into();
                T::read(path).await.ok()
            }
        };
        log::trace!("Config ready: {:?}", config);
        Ok(config)
    }
}

#[async_trait]
pub trait WritableConfig: Config + Serialize + Sync {
    async fn write(&self, path: PathBuf) -> Result<(), Error> {
        let mut file = File::create(path).await?;
        let data = toml::to_vec(self)?;
        file.write_all(&data).await?;
        Ok(())
    }

    async fn drop_file(path: PathBuf) -> Result<(), Error> {
        remove_file(path).await?;
        Ok(())
    }
}

pub struct WriteConfigFile<T> {
    path: PathBuf,
    config: T,
}

impl<T> WriteConfigFile<T> {
    pub fn new(path: PathBuf, config: T) -> Self {
        Self { path, config }
    }
}

#[async_trait]
impl<T: WritableConfig> LiteTask for WriteConfigFile<T> {
    type Output = ();

    async fn interruptable_routine(mut self) -> Result<Self::Output, Error> {
        log::trace!("Storing config to {:?}: {:?}", self.path, self.config);
        self.config.write(self.path).await?;
        Ok(())
    }
}

pub struct DropConfigFile<T> {
    path: PathBuf,
    _config: PhantomData<T>,
}

impl<T> DropConfigFile<T> {
    pub fn new(path: PathBuf) -> Self {
        Self {
            path,
            _config: PhantomData,
        }
    }
}

#[async_trait]
impl<T: WritableConfig> LiteTask for DropConfigFile<T> {
    type Output = ();

    async fn interruptable_routine(mut self) -> Result<Self::Output, Error> {
        log::trace!("Drop config file: {:?}", self.path);
        T::drop_file(self.path).await?;
        Ok(())
    }
}
