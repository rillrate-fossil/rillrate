use crate::base::list_flow::{ListFlowSpec, ListFlowState};
use rill_protocol::io::provider::Path;
use serde::{Deserialize, Serialize};
use std::collections::BTreeSet;

pub type GroupsListState = ListFlowState<GroupsListSpec>;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum GroupsListUpdate {
    JoinGroup { path: Path },
    LeaveGroup { path: Path },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GroupsListSpec;

impl ListFlowSpec for GroupsListSpec {
    type Id = Path;
    type Record = BTreeSet<Path>;
    type Action = ();
    type Update = GroupsListUpdate;

    fn path() -> Path {
        "rillrate.manifest.groups_list".parse().unwrap()
    }

    fn update_record(record: &mut Self::Record, update: Self::Update) {
        match update {
            GroupsListUpdate::JoinGroup { path } => {
                record.insert(path);
            }
            GroupsListUpdate::LeaveGroup { path } => {
                record.remove(&path);
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
