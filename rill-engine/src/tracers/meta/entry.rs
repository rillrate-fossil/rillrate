use crate::tracers::tracer::{DataEnvelope, Tracer, TracerEvent, TracerState};
use derive_more::{Deref, DerefMut};
use rill_protocol::provider::{RillEvent, EntryId, Path};
use std::collections::{HashMap, HashSet};

#[derive(Debug)]
pub enum EntryRecord {
    AddProvider { name: EntryId },
    RemoveProvider { name: EntryId },
}

#[derive(Debug, Default)]
pub struct EntryState {
    map: HashMap<EntryId, HashSet<Path>>,
}

/// This tracer sends text messages.
#[derive(Debug, Deref, DerefMut)]
pub struct EntryTracer {
    tracer: Tracer<EntryRecord>,
}

impl TracerState for EntryState {
    type Item = EntryRecord;

    fn aggregate(
        &mut self,
        items: Vec<DataEnvelope<Self::Item>>,
        outgoing: Option<&mut Vec<RillEvent>>,
    ) {
    }

    fn make_snapshot(&self) -> Vec<RillEvent> {
        Vec::new()
    }
}

impl TracerEvent for EntryRecord {
    type State = EntryState;
}
