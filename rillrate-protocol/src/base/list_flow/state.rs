use rill_protocol::flow::core::{DataFraction, Flow};
use rill_protocol::io::provider::{Path, StreamType};
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;
use std::hash::Hash;

pub trait ListFlowSpec: DataFraction {
    type Id: DataFraction + Ord + Hash;
    type Record: DataFraction;
    type Action: DataFraction;
    type Update: DataFraction; // aka `Event`, but inner

    fn path() -> Path;

    fn update_record(record: &mut Self::Record, update: Self::Update);
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ListFlowState<T: ListFlowSpec> {
    // TODO: Use `ListFlowSnapshot` here instead.
    #[serde(with = "vectorize")]
    pub records: BTreeMap<T::Id, T::Record>,
}

#[allow(clippy::new_without_default)]
impl<T: ListFlowSpec> ListFlowState<T> {
    pub fn new() -> Self {
        Self {
            records: BTreeMap::new(),
        }
    }
}

impl<T: ListFlowSpec> Flow for ListFlowState<T> {
    type Action = ListActionEnvelope<T>;
    type Event = ListEventEnvelope<T>;

    fn stream_type() -> StreamType {
        StreamType::from(module_path!())
    }

    fn apply(&mut self, event: Self::Event) {
        match event {
            ListEventEnvelope::SingleRecord { id, update } => match update {
                ListFlowEvent::AddRecord { record } => {
                    self.records.insert(id, record);
                }
                ListFlowEvent::UpdateRecord { update } => {
                    if let Some(record) = self.records.get_mut(&id) {
                        T::update_record(record, update);
                    } else {
                        log::error!("List record with {:?} not found.", id);
                    }
                }
                ListFlowEvent::RemoveRecord => {
                    self.records.remove(&id);
                }
            },
            ListEventEnvelope::FullSnapshot { records } => {
                self.records = records;
            }
            ListEventEnvelope::Clear => {
                self.records.clear();
            }
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ListActionEnvelope<T: ListFlowSpec> {
    pub id: T::Id,
    pub action: T::Action,
}

impl<T: ListFlowSpec> From<(T::Id, T::Action)> for ListActionEnvelope<T> {
    fn from((id, action): (T::Id, T::Action)) -> Self {
        Self { id, action }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ListEventEnvelope<T: ListFlowSpec> {
    SingleRecord {
        id: T::Id,
        #[serde(bound = "")]
        update: ListFlowEvent<T>,
    },
    FullSnapshot {
        #[serde(with = "vectorize")]
        records: BTreeMap<T::Id, T::Record>,
    },
    Clear,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ListFlowEvent<T: ListFlowSpec> {
    AddRecord { record: T::Record },
    UpdateRecord { update: T::Update },
    RemoveRecord,
}
