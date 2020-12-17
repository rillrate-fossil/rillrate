use super::provider::Provider;
use crate::protocol::{Path, RillData, StreamType};
use derive_more::{Deref, DerefMut};

#[derive(Debug, Deref, DerefMut)]
pub struct CounterProvider {
    provider: Provider,
}

impl CounterProvider {
    pub fn new(path: Path) -> Self {
        let provider = Provider::new(path, StreamType::CounterStream);
        Self { provider }
    }

    pub fn inc(&self, delta: u64) {
        todo!();
    }
}
