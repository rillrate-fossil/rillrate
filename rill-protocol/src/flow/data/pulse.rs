use crate::flow::core::{Flow, TimedEvent};
// TODO: Join `Frame` and `Range` into a single module.
use crate::frame::Frame;
use crate::io::provider::StreamType;
use crate::range::Range;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PulsePoint {
    pub value: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PulseState {
    // IMMUTABLE:
    pub range: Option<Range>,

    // MUTABLE:
    pub frame: Frame<TimedEvent<PulsePoint>>,
    /// Intermediate counter value. Not available for changing!!!
    value: f64,
}

impl PulseState {
    pub fn new(range: Option<Range>, depth: Option<u32>) -> Self {
        let depth = depth.unwrap_or(128);
        Self {
            range,
            // TODO: Use duration for removing obsolete values instead
            frame: Frame::new(depth),
            value: 0.0,
        }
    }

    /*
    pub fn range(&self) -> Cow<Range> {
        self.range.as_ref().map(Cow::Borrowed).unwrap_or_else(|| {
            // TODO: Calc min and max from Frame
            Cow::Owned(Range::new(0.0, 100.0))
        })
    }

    pub fn pct(&self) -> Pct {
        Pct::from_range(self.value, self.range().as_ref())
    }
    */
}

impl Flow for PulseState {
    type Action = ();
    type Event = PulseEvent;

    fn stream_type() -> StreamType {
        StreamType::from("rillrate.data.pulse.v0")
    }

    fn apply(&mut self, event: TimedEvent<Self::Event>) {
        match event.event {
            PulseEvent::Inc(delta) => {
                self.value += delta;
            }
            PulseEvent::Dec(delta) => {
                self.value -= delta;
            }
            PulseEvent::Set(value) => {
                self.value = value;
            }
        }
        // Use the clamped value if a range set, but don't affect the state.
        let mut value = self.value;
        if let Some(range) = self.range.as_ref() {
            range.clamp(&mut value);
        }
        let point = PulsePoint { value };
        let timed_event = TimedEvent {
            timestamp: event.timestamp,
            event: point,
        };
        self.frame.insert_pop(timed_event);
    }
}

pub type PulseDelta = Vec<TimedEvent<PulseEvent>>;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum PulseEvent {
    Inc(f64),
    Dec(f64),
    Set(f64),
}
