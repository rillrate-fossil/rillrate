use crate::base::list_flow::{ListFlowSpec, ListFlowState};
use rill_protocol::io::provider::{Description, Path};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DescriptionsListSpec;

impl ListFlowSpec for DescriptionsListSpec {
    type Id = Path;
    type Record = Description;
    type Action = ();
    type Update = ();

    fn path() -> Path {
        "rillrate.manifest.descriptions_list".parse().unwrap()
    }

    fn update_record(_record: &mut Self::Record, _update: Self::Update) {
        log::error!("Inner updates not supported to DescriptionsList");
    }
}

pub type DescriptionsListState = ListFlowState<DescriptionsListSpec>;
