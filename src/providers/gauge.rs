use super::provider::Provider;
use crate::protocol::{Path, RillData, StreamType};
use derive_more::{Deref, DerefMut};
use std::sync::atomic::{AtomicU64, Ordering};
use std::time::SystemTime;

#[derive(Debug, Deref, DerefMut)]
pub struct GaugeProvider {
    #[deref]
    #[deref_mut]
    provider: Provider,
    value: AtomicU64,
}

impl GaugeProvider {
    pub fn new(path: Path) -> Self {
        let provider = Provider::new(path, StreamType::GaugeStream);
        Self {
            provider,
            value: AtomicU64::new(0),
        }
    }

    pub fn inc(&self, delta: u64, timestamp: Option<SystemTime>) {
        let prev = self.value.fetch_add(delta, Ordering::SeqCst);
        let data = RillData::CounterRecord {
            value: prev + delta,
        };
        self.provider.send(data, timestamp);
    }

    pub fn sub(&self, delta: u64, timestamp: Option<SystemTime>) {
        let prev = self.value.fetch_sub(delta, Ordering::SeqCst);
        let data = RillData::CounterRecord {
            value: prev - delta,
        };
        self.provider.send(data, timestamp);
    }

    pub fn set(&self, value: u64, timestamp: Option<SystemTime>) {
        self.value.store(value, Ordering::SeqCst);
        let data = RillData::CounterRecord { value };
        self.provider.send(data, timestamp);
    }
}
