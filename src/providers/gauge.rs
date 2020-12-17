use super::provider::Provider;
use crate::protocol::{Path, RillData, StreamType};
use derive_more::{Deref, DerefMut};
use std::sync::{Mutex, MutexGuard};
use std::time::SystemTime;

#[derive(Debug, Deref, DerefMut)]
pub struct GaugeProvider {
    #[deref]
    #[deref_mut]
    provider: Provider,
    value: Mutex<f64>,
}

impl GaugeProvider {
    pub fn new(path: Path) -> Self {
        let provider = Provider::new(path, StreamType::GaugeStream);
        Self {
            provider,
            value: Mutex::new(0.0),
        }
    }

    fn lock(&self) -> Option<MutexGuard<'_, f64>> {
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

    pub fn inc(&self, delta: f64, timestamp: Option<SystemTime>) {
        if let Some(mut value) = self.lock() {
            *value += delta;
            let data = RillData::CounterRecord { value: *value };
            self.provider.send(data, timestamp);
        }
    }

    pub fn sub(&self, delta: f64, timestamp: Option<SystemTime>) {
        if let Some(mut value) = self.lock() {
            *value -= delta;
            let data = RillData::CounterRecord { value: *value };
            self.provider.send(data, timestamp);
        }
    }

    pub fn set(&self, new_value: f64, timestamp: Option<SystemTime>) {
        if let Some(mut value) = self.lock() {
            *value = new_value;
            let data = RillData::CounterRecord { value: *value };
            self.provider.send(data, timestamp);
        }
    }
}
