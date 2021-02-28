use crate::tracers::tracer::{DataEnvelope, Tracer, TracerEvent, TracerState};
use derive_more::{Deref, DerefMut};
use rill_protocol::provider::{
    Description, EntryId, EntryUpdate, Path, RillData, RillEvent, StreamType, Timestamp,
};
use std::collections::HashMap;

#[derive(Debug)]
pub enum EntryRecord {
    AddProvider { name: EntryId },
    RemoveProvider { name: EntryId },
}

#[derive(Debug, Default)]
pub struct EntryState {
    map: HashMap<EntryId, Timestamp>,
}

impl TracerState for EntryState {
    type Item = EntryRecord;

    fn aggregate(
        &mut self,
        items: Vec<DataEnvelope<Self::Item>>,
        _outgoing: Option<&mut Vec<RillEvent>>,
    ) {
        log::trace!("EntryState incoiming: {:?}", items);
        log::error!("EntryState aggregation not implemented yet.");
    }

    fn make_snapshot(&self) -> Vec<RillEvent> {
        self.map
            .iter()
            .map(|(name, ts)| {
                let update = EntryUpdate::Add { name: name.clone() };
                let data = RillData::EntryUpdate(update);
                RillEvent {
                    timestamp: ts.clone(),
                    data,
                }
            })
            .collect()
    }
}

impl TracerEvent for EntryRecord {
    type State = EntryState;
}

/// This tracer sends entries changes.
#[derive(Debug, Deref, DerefMut, Clone)]
pub struct EntryTracer {
    tracer: Tracer<EntryRecord>,
}

impl EntryTracer {
    /// Create a new instance of the `Tracer`.
    pub fn new(path: Path) -> Self {
        let info = format!("{} entries", path);
        let description = Description {
            path,
            info,
            stream_type: StreamType::EntryStream,
        };
        let tracer = Tracer::new(description);
        Self { tracer }
    }

    /// Registers a new provider
    pub fn add_provider(&self, entry_id: EntryId) -> ProviderRecord {
        let data = EntryRecord::AddProvider {
            name: entry_id.clone(),
        };
        self.tracer.send(data, None);
        ProviderRecord {
            tracer: self.tracer.clone(),
            name: Some(entry_id),
        }
    }
}

pub struct ProviderRecord {
    tracer: Tracer<EntryRecord>,
    name: Option<EntryId>,
}

impl ProviderRecord {
    fn remove_provider(&mut self) {
        if let Some(name) = self.name.take() {
            let data = EntryRecord::AddProvider { name };
            self.tracer.send(data, None);
        } else {
            log::error!("Attempt to remove provider twice.");
        }
    }
}

impl Drop for ProviderRecord {
    fn drop(&mut self) {
        self.remove_provider();
    }
}
