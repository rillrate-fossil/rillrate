use super::provider::Provider;
use crate::protocol::{Path, RillData, StreamType};
use derive_more::{Deref, DerefMut};
use std::sync::Mutex;
use std::time::SystemTime;

#[derive(Debug, Deref, DerefMut)]
pub struct CounterProvider {
    #[deref]
    #[deref_mut]
    provider: Provider,
    value: Mutex<f64>,
}

impl CounterProvider {
    pub fn new(path: Path) -> Self {
        let provider = Provider::new(path, StreamType::CounterStream);
        Self {
            provider,
            value: Mutex::new(0.0),
        }
    }

    pub fn inc(&self, delta: f64, timestamp: Option<SystemTime>) {
        match self.value.lock() {
            Ok(mut value) => {
                *value += delta;
                let data = RillData::CounterRecord { value: *value };
                self.provider.send(data, timestamp);
            }
            Err(err) => {
                log::error!("Can't lock value of counter: {}", self.provider.path());
            }
        }
    }
}
