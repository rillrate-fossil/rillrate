use crate::base::stat_flow::{StatFlowSpec, StatFlowState};
use rill_protocol::io::provider::Path;
use serde::{Deserialize, Serialize};
use std::time::Duration;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CounterStatSpec {
    pub id: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct CounterStat {
    total: i64,
}

impl StatFlowSpec for CounterStatSpec {
    type Stat = CounterStat;
    type Delta = CounterStatDelta;

    fn path(&self) -> Path {
        format!("rillrate.live_data.counter.{}", self.id)
            .parse()
            .unwrap()
    }

    fn interval(&self) -> Duration {
        Duration::from_millis(1_000)
    }

    fn apply(stat: &mut Self::Stat, delta: Self::Delta) {
        match delta {
            CounterStatDelta::Inc { delta } => {
                stat.total += delta;
            }
        }
        // TODO
    }
}

pub type CounterStatState = StatFlowState<CounterStatSpec>;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum CounterStatDelta {
    Inc { delta: i64 },
}
