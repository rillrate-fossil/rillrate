use serde::{Deserialize, Serialize};

#[derive(Deserialize, Serialize, Debug, Clone, Default)]
pub struct Avg {
    counter: u64,
    sum: f64,
}

impl Avg {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn add(&mut self, value: f64) {
        self.counter += 1;
        self.sum += value;
    }

    pub fn value(&self) -> f64 {
        if self.counter == 0 {
            0.0
        } else {
            self.sum / self.counter as f64
        }
    }
}
