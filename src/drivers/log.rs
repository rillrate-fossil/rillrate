use crate::ProviderCell;
use log::{Log, Metadata, Record};
use std::collections::HashMap;
use std::sync::RwLock;

pub struct LogDriver {
    providers: RwLock<HashMap<String, ProviderCell>>,
}

impl Log for LogDriver {
    fn enabled(&self, metadata: &Metadata<'_>) -> bool {
        if let Some(provider) = self.providers.read().unwrap().get(metadata.target()) {
            provider.is_active()
        } else {
            // TODO: Create a provider...
            false
        }
    }

    fn log(&self, record: &Record<'_>) {
        todo!();
    }

    fn flush(&self) {}
}
