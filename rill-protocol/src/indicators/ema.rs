use serde::{Deserialize, Serialize};

#[derive(Deserialize, Serialize, Debug, Clone)]
pub struct Ema {
    ema: f64,
    k: f64,
}

impl Ema {
    pub fn new(initial: f64, period: u32) -> Self {
        let k = 2.0 / (period as f64 + 1.0);
        Self { ema: initial, k }
    }

    pub fn update(&mut self, value: f64) {
        self.ema = value * self.k + self.ema * (1.0 - self.k);
    }

    pub fn value(&self) -> f64 {
        self.ema
    }
}
