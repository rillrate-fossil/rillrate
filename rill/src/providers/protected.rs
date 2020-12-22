use super::provider::Provider;
use crate::protocol::{Path, StreamType};
use derive_more::{Deref, DerefMut};
use std::sync::{Mutex, MutexGuard};

#[derive(Debug, Deref, DerefMut)]
pub struct ProtectedProvider<T> {
    #[deref]
    #[deref_mut]
    provider: Provider,
    value: Mutex<T>,
}

impl<T> ProtectedProvider<T> {
    pub(super) fn new(path: Path, stream_type: StreamType, data: T) -> Self {
        let provider = Provider::new(path, stream_type);
        Self {
            provider,
            value: Mutex::new(data),
        }
    }

    pub(super) fn lock(&self) -> Option<MutexGuard<'_, T>> {
        match self.value.lock() {
            Ok(value) => Some(value),
            Err(err) => {
                log::error!(
                    "Can't lock protected data of {}: {}",
                    self.provider.path(),
                    err
                );
                None
            }
        }
    }
}
