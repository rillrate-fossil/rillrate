use crate::base::list_flow::{ListFlowSpec, ListFlowState};
use rill_protocol::io::provider::{EntryId, Path};
use serde::{Deserialize, Serialize};
use std::collections::BTreeSet;

pub type GroupsListState = ListFlowState<GroupsListSpec>;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum GroupsListUpdate {
    JoinGroup { entry_id: EntryId },
    LeaveGroup { entry_id: EntryId },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GroupsListSpec;

impl GroupsListSpec {
    pub fn path() -> Path {
        "rillrate.manifest.groups_list".parse().unwrap()
    }
}

impl ListFlowSpec for GroupsListSpec {
    type Id = EntryId;
    type Record = BTreeSet<EntryId>;
    type Action = ();
    type Update = GroupsListUpdate;

    fn update_record(record: &mut Self::Record, update: Self::Update) {
        match update {
            GroupsListUpdate::JoinGroup { entry_id } => {
                record.insert(entry_id);
            }
            GroupsListUpdate::LeaveGroup { entry_id } => {
                record.remove(&entry_id);
            }
        }
    }

    fn no_record_fallback(_id: &Self::Id) -> Option<Self::Record> {
        Some(BTreeSet::new())
    }

    fn is_spent(record: &Self::Record) -> bool {
        record.is_empty()
    }
}
