use crate::range::{Bound, Range};
use rill_protocol::flow::core::Flow;
use rill_protocol::io::provider::StreamType;
use serde::{Deserialize, Serialize};

// TODO: Move builder to the separate module?
pub struct GaugeOpts {
    pub min: Option<f64>,
    pub lower: Option<bool>,
    pub max: Option<f64>,
    pub higher: Option<bool>,
}

impl From<GaugeOpts> for GaugeSpec {
    fn from(opts: GaugeOpts) -> Self {
        Self {
            range: Range {
                min: Bound::from_options(opts.min, opts.lower),
                max: Bound::from_options(opts.max, opts.higher),
            },
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct GaugeSpec {
    pub range: Range,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GaugeState {
    pub spec: GaugeSpec,
    pub value: Option<f64>,
    pub abs_min: f64,
    pub abs_max: f64,
}

impl From<GaugeSpec> for GaugeState {
    fn from(spec: GaugeSpec) -> Self {
        Self {
            spec,
            value: None,
            abs_min: f64::MAX,
            abs_max: f64::MIN,
        }
    }
}

impl Flow for GaugeState {
    type Action = ();
    type Event = GaugeEvent;

    fn stream_type() -> StreamType {
        StreamType::from(module_path!())
    }

    fn apply(&mut self, event: Self::Event) {
        match event {
            GaugeEvent::Set { value } => {
                self.value = Some(value);
                if value < self.abs_min {
                    self.abs_min = value;
                }
                if value > self.abs_max {
                    self.abs_max = value;
                }
            }
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum GaugeEvent {
    Set { value: f64 },
}
