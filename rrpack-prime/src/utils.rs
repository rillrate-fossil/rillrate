use rill_protocol::timed_frame::TimedFrame;

pub fn new_tf<T>(secs: i64) -> TimedFrame<T> {
    TimedFrame::new((secs + 1) * 1_000)
}
