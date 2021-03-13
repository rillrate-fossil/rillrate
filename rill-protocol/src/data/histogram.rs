use super::{Metric, TimedEvent};
use crate::io::codec::vectorize;
use crate::io::provider::StreamType;
use ordered_float::OrderedFloat;
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

#[derive(Debug)]
pub struct HistogramMetric;

impl Metric for HistogramMetric {
    type State = HistogramState;
    type Event = HistogramEvent;

    fn stream_type() -> StreamType {
        StreamType::from("rillrate.histogram.v0")
    }

    fn apply(state: &mut Self::State, event: TimedEvent<Self::Event>) {
        match event.event {
            HistogramEvent::Add(amount) => {
                state.total.add(amount);
                let expected = OrderedFloat::from(amount);
                for (level, stat) in &mut state.buckets {
                    if &expected <= level {
                        stat.add(amount);
                    }
                }
            }
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Stat {
    pub sum: f64,
    pub count: u64,
}

impl Default for Stat {
    fn default() -> Self {
        Self { sum: 0.0, count: 0 }
    }
}

impl Stat {
    fn add(&mut self, value: f64) {
        self.sum += value;
        self.count += 1;
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HistogramState {
    #[serde(with = "vectorize")]
    pub buckets: BTreeMap<OrderedFloat<f64>, Stat>,
    pub total: Stat,
}

impl HistogramState {
    pub fn new(levels: &[f64]) -> Self {
        let mut buckets: BTreeMap<_, _> = levels
            .iter()
            .map(|level| (OrderedFloat::from(*level), Stat::default()))
            .collect();
        let inf_level = OrderedFloat::from(f64::INFINITY);
        if !buckets.contains_key(&inf_level) {
            buckets.insert(inf_level, Stat::default());
        }
        Self {
            buckets,
            total: Stat::default(),
        }
    }
}

pub type HistogramDelta = Vec<TimedEvent<HistogramEvent>>;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum HistogramEvent {
    Add(f64),
}
