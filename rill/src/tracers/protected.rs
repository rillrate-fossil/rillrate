use super::tracer::{Tracer, TracerType};
use derive_more::{Deref, DerefMut};
use rill_protocol::provider::Description;
use std::sync::{Mutex, MutexGuard};

/// Special wrapper to give shared access to the `Tracer`.
#[derive(Debug, Deref, DerefMut)]
pub struct ProtectedTracer<T> {
    #[deref]
    #[deref_mut]
    tracer: Tracer,
    value: Mutex<T>,
}

impl<T> ProtectedTracer<T> {
    pub(super) fn new(description: Description, data: T, active: bool) -> Self {
        let mut types = vec![TracerType::Realtime];
        if active {
            types.push(TracerType::Snapshot);
        }
        let tracer = Tracer::new(description, &types);
        Self {
            tracer,
            value: Mutex::new(data),
        }
    }

    pub(super) fn lock(&self) -> Option<MutexGuard<'_, T>> {
        match self.value.lock() {
            Ok(value) => Some(value),
            Err(err) => {
                log::error!(
                    "Can't lock protected data of {}: {}",
                    self.tracer.path(),
                    err
                );
                None
            }
        }
    }
}
