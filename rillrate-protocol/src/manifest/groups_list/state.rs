use crate::base::list_flow::{ListFlowSpec, ListFlowState};
use rill_protocol::io::provider::Path;
use serde::{Deserialize, Serialize};
use std::collections::BTreeSet;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GroupsListSpec;

impl ListFlowSpec for GroupsListSpec {
    type Id = Path;
    type Record = BTreeSet<Path>;
    type Action = ();
    type Update = ();

    fn path() -> Path {
        "rillrate.manifest.groups_list".parse().unwrap()
    }

    fn update_record(_record: &mut Self::Record, _update: Self::Update) {
        // TODO: Impement it
        log::error!("Inner updates not supported to GroupsList");
    }
}

pub type GroupsListState = ListFlowState<GroupsListSpec>;
