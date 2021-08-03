use crate::base::list_flow::{ListFlowSpec, ListFlowState};
use rill_protocol::io::provider::{Description, Path};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DescriptionsFlowSpec;

impl ListFlowSpec for DescriptionsFlowSpec {
    type Id = Path;
    type Record = Description;
    type Action = ();
    type Update = ();

    fn path() -> Path {
        "rillrate.manifest.tracers_flow".parse().unwrap()
    }

    fn update_record(_record: &mut Self::Record, _update: Self::Update) {
        log::error!("Inner updates not supported to DescriptionsFlow");
    }
}

pub type DescriptionsFlowState = ListFlowState<DescriptionsFlowSpec>;
