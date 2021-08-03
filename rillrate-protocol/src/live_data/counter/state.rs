use crate::base::stat_flow::{StatFlowSpec, StatFlowState};
use rill_protocol::io::provider::{EntryId, Path};
use serde::{Deserialize, Serialize};
use std::time::Duration;

pub type CounterStatState = StatFlowState<CounterStatSpec>;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CounterStatSpec {
    pub name: EntryId,
    pub pull_ms: Option<u64>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct CounterStat {
    pub total: i64,
}

impl StatFlowSpec for CounterStatSpec {
    type Stat = CounterStat;
    type Delta = i64;

    fn path(&self) -> Path {
        // TODO: Improve that
        format!("rillrate.live_data.counter.{}", self.name)
            .parse()
            .unwrap()
    }

    fn interval(&self) -> Option<Duration> {
        self.pull_ms.map(Duration::from_millis)
    }

    fn apply(stat: &mut Self::Stat, delta: Self::Delta) {
        stat.total += delta;
    }
}
