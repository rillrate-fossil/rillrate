use crate::base::frame_flow::{FrameFlowSpec, FrameFlowState};
use rill_protocol::io::provider::{EntryId, Path, StreamType};
use serde::{Deserialize, Serialize};

pub type PulseFrameState = FrameFlowState<PulseFrameSpec>;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PulseFrameSpec {
    pub name: EntryId,
}

impl FrameFlowSpec for PulseFrameSpec {
    // TODO: Add range here
    type Info = ();
    type Frame = f32;

    fn path(&self) -> Path {
        // TODO: Improve that
        format!("rillrate.live_data.pulse.{}", self.name)
            .parse()
            .unwrap()
    }

    fn retain_secs() -> u32 {
        31
    }
}
