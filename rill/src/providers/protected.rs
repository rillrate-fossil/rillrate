use super::provider::Provider;
use derive_more::{Deref, DerefMut};
use rill_protocol::provider::Description;
use std::sync::{Mutex, MutexGuard};

/// Special wrapper to give shared access to the `Provider`.
#[derive(Debug, Deref, DerefMut)]
pub struct ProtectedProvider<T> {
    #[deref]
    #[deref_mut]
    provider: Provider,
    value: Mutex<T>,
}

impl<T> ProtectedProvider<T> {
    pub(super) fn new(description: Description, data: T) -> Self {
        let provider = Provider::new(description);
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
