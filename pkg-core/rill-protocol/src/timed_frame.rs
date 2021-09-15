use crate::flow::core::TimedEvent;
use derive_more::Deref;
use serde::{Deserialize, Serialize};
use std::collections::VecDeque;

static DEFAULT_DEPTH: i64 = 60_000;

#[derive(Debug, Clone, Serialize, Deserialize, Deref)]
pub struct TimedFrame<T> {
    depth_ms: i64,
    #[deref]
    frame: VecDeque<TimedEvent<T>>,
}

impl<T> Default for TimedFrame<T> {
    fn default() -> Self {
        Self::new(DEFAULT_DEPTH)
    }
}

impl<T> TimedFrame<T> {
    pub fn new(depth_ms: i64) -> Self {
        Self {
            depth_ms,
            frame: VecDeque::new(),
        }
    }

    pub fn insert_pop(&mut self, item: TimedEvent<T>) {
        while let Some(front) = self.frame.front() {
            if (item.timestamp.0 - front.timestamp.0) >= self.depth_ms {
                self.frame.pop_front();
            } else {
                break;
            }
        }
        self.frame.push_back(item);
    }

    pub fn depth_ms(&self) -> i64 {
        self.depth_ms
    }

    pub fn clear(&mut self) {
        self.frame.clear()
    }
}
