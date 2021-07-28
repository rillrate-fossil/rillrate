use crate::actors::provider_session::ProviderLink;
//use rate_meta::flow::entry::ENTRIES;
//use rate_meta::tracer::entry::EntryTracer;
use rill_protocol::io::provider::{Description, Path};
use rill_protocol::pathfinder::{Pathfinder, Record};
use std::sync::Arc;
use tokio::sync::RwLock;

/// The `Path` that has validated.
pub struct ValidPath(pub Path);

pub struct Occupied {
    pub path: Path,
}

pub struct WasEmpty {
    pub path: Path,
}

#[derive(Debug, Clone)]
pub struct Registry {
    inner: Arc<RwLock<RegistryInner>>,
}

impl Registry {
    pub fn new() -> Self {
        let inner = RegistryInner::new();
        Self {
            inner: Arc::new(RwLock::new(inner)),
        }
    }

    pub async fn register_provider(
        &mut self,
        path: Path,
        _description: Description,
        provider: ProviderLink,
    ) -> Result<ProviderEntry, Occupied> {
        log::debug!("Registering provider: {}", path);
        let mut inner = self.inner.write().await;
        let record = inner.providers.dig(path.clone());
        if !record.has_link() {
            record.set_link(provider);
            //inner.tracer.add(path.clone(), description);
            let entry = ProviderEntry {
                inner: self.inner.clone(),
                path,
            };
            Ok(entry)
        } else {
            Err(Occupied { path })
        }
    }

    pub async fn find_provider(&self, path: &ValidPath) -> Option<(ProviderLink, Path)> {
        let inner = self.inner.read().await;
        let discovered = inner.providers.discover(&path.0);
        discovered
            .record
            .get_link()
            .map(ProviderLink::clone)
            .map(move |link| (link, discovered.remained_path))
    }
}

#[derive(Debug)]
pub struct RegistryInner {
    providers: Pathfinder<ProviderLink>,
    //tracer: EntryTracer,
}

impl RegistryInner {
    fn new() -> Self {
        let providers = Pathfinder::new();
        //let tracer = EntryTracer::new(ENTRIES.root());
        Self {
            providers,
            //tracer,
        }
    }
}

pub struct ProviderEntry {
    inner: Arc<RwLock<RegistryInner>>,
    path: Path,
}

impl ProviderEntry {
    pub async fn unregister_provider(self) -> Result<(), WasEmpty> {
        let mut inner = self.inner.write().await;
        let path = self.path.clone();
        let link = inner.providers.find_mut(&path).and_then(Record::take_link);
        if link.is_some() {
            //inner.tracer.del(self.path);
            Ok(())
        } else {
            Err(WasEmpty { path: self.path })
        }
    }
}
