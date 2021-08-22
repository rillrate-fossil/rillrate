use ordered_float::OrderedFloat;
use rill_protocol::flow::core::Flow;
use rill_protocol::io::provider::StreamType;
use rill_protocol::range::Pct;
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HistogramSpec {
    pub levels: Vec<f64>,
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

    /* TODO: Use later for sliding window
    fn del(&mut self, value: f64) {
        self.sum -= value;
        self.count -= 1;
    }
    */
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HistogramState {
    #[serde(with = "vectorize")]
    pub buckets: BTreeMap<OrderedFloat<f64>, Stat>,
    pub total: Stat,
}

impl From<HistogramSpec> for HistogramState {
    fn from(spec: HistogramSpec) -> Self {
        let mut buckets: BTreeMap<_, _> = spec
            .levels
            .iter()
            .map(|level| (OrderedFloat::from(*level), Stat::default()))
            .collect();
        let inf_level = OrderedFloat::from(f64::INFINITY);
        buckets.entry(inf_level).or_default();
        Self {
            buckets,
            total: Stat::default(),
        }
    }
}

impl HistogramState {
    pub fn bars(&self) -> impl Iterator<Item = Bar> + '_ {
        let total = self.total.sum;
        self.buckets.iter().map(move |(level, stat)| Bar {
            level: *level,
            count: stat.count,
            pct: Pct::from_div(stat.sum, total),
        })
    }
}

impl Flow for HistogramState {
    type Action = ();
    type Event = HistogramEvent;

    fn stream_type() -> StreamType {
        StreamType::from(module_path!())
    }

    fn apply(&mut self, event: Self::Event) {
        match event {
            HistogramEvent::Add(amount) => {
                self.total.add(amount);
                let expected = OrderedFloat::from(amount);
                for (level, stat) in &mut self.buckets {
                    if &expected <= level {
                        stat.add(amount);
                        break;
                    }
                }
            }
        }
    }
}

pub struct Bar {
    pub level: OrderedFloat<f64>,
    pub count: u64,
    pub pct: Pct,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum HistogramEvent {
    Add(f64),
}
