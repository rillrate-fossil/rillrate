use crate::base::list_flow::{ListFlowSpec, ListFlowState};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BoardSpec;

impl ListFlowSpec for BoardSpec {
    type Id = String;
    type Record = String;
    type Action = ();
    type Update = ();

    fn update_record(_record: &mut Self::Record, _update: Self::Update) {
        log::error!("Inner updates are not supported for Board");
    }
}

pub type BoardState = ListFlowState<BoardSpec>;
