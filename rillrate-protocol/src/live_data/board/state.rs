use crate::base::list_flow::{ListFlowSpec, ListFlowState};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BoardListSpec;

impl ListFlowSpec for BoardListSpec {
    type Id = String;
    type Record = String;
    type Action = ();
    type Update = ();

    fn update_record(_record: &mut Self::Record, _update: Self::Update) {
        log::error!("Inner updates not supported to BoardList");
    }
}

pub type BoardListState = ListFlowState<BoardListSpec>;
