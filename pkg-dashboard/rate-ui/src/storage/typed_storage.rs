use anyhow::Error;
use serde::{de::DeserializeOwned, Serialize};
use std::marker::PhantomData;
use yew::format::Json;
use yew::services::storage::{Area, StorageService};

pub trait Storable: DeserializeOwned + Serialize {
    fn key() -> &'static str;
}

#[derive(Debug)]
pub struct TypedStorage<T> {
    storage: StorageService,
    _storable: PhantomData<T>,
}

impl<T> Default for TypedStorage<T> {
    fn default() -> Self {
        Self::new()
    }
}

impl<T> TypedStorage<T> {
    pub fn new() -> Self {
        let storage = StorageService::new(Area::Local).expect("no access to storage");
        Self {
            storage,
            _storable: PhantomData,
        }
    }
}

impl<T: Storable> TypedStorage<T> {
    pub fn restore(&self) -> Result<T, Error> {
        let Json(res) = self.storage.restore(T::key());
        res
    }

    pub fn store(&mut self, value: &T) {
        self.storage.store(T::key(), Json(value));
    }
}
