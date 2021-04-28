use crate::flow::core::{Flow, TimedEvent};
use crate::frame::Frame;
use crate::io::provider::StreamType;
use crate::range::Pct;
use ordered_float::OrderedFloat;
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

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

    fn del(&mut self, value: f64) {
        self.sum -= value;
        self.count -= 1;
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HistogramState {
    #[serde(with = "vectorize")]
    pub buckets: BTreeMap<OrderedFloat<f64>, Stat>,
    pub total: Stat,
    frame: Option<Frame<f64>>,
}

impl HistogramState {
    pub fn new(levels: Vec<f64>, window: Option<u32>) -> Self {
        let mut buckets: BTreeMap<_, _> = levels
            .iter()
            .map(|level| (OrderedFloat::from(*level), Stat::default()))
            .collect();
        let inf_level = OrderedFloat::from(f64::INFINITY);
        buckets.entry(inf_level).or_default();
        Self {
            buckets,
            total: Stat::default(),
            frame: window.map(Frame::new),
        }
    }

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
        StreamType::from("rillrate.data.histogram.v0")
    }

    fn apply(&mut self, event: TimedEvent<Self::Event>) {
        match event.event {
            HistogramEvent::Add(amount) => {
                self.total.add(amount);
                let expected = OrderedFloat::from(amount);
                for (level, stat) in &mut self.buckets {
                    if &expected <= level {
                        stat.add(amount);
                        break;
                    }
                }

                // If sliding window is active
                if let Some(frame) = self.frame.as_mut() {
                    if let Some(amount) = frame.insert_pop(amount) {
                        self.total.del(amount);
                        let expected = OrderedFloat::from(amount);
                        for (level, stat) in &mut self.buckets {
                            if &expected <= level {
                                stat.del(amount);
                                break;
                            }
                        }
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

pub type HistogramDelta = Vec<TimedEvent<HistogramEvent>>;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum HistogramEvent {
    Add(f64),
}
