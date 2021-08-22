pub mod frame_flow;
//pub mod list_flow;
//pub mod meta_flow;
//pub mod stat_flow;

use rill_protocol::timed_frame::TimedFrame;

pub fn new_tf<T>(secs: i64) -> TimedFrame<T> {
    TimedFrame::new((secs + 1) * 1_000)
}
