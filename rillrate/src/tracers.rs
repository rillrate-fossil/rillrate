use rill::tracers::{CounterTracer, GaugeTracer, LogTracer, Tracer};
use std::ops::Deref;
use std::sync::Arc;

macro_rules! deref_tracer {
    ($t:ty) => {
        impl Deref for $t {
            type Target = Tracer;

            fn deref(&self) -> &Self::Target {
                &self.tracer
            }
        }
    };
}

#[derive(Debug, Clone)]
pub struct Counter {
    tracer: Arc<CounterTracer>,
}

deref_tracer!(Counter);

impl Counter {
    /// Increments value by the sepcific delta.
    pub fn inc(&self, delta: f64) {
        self.tracer.inc(delta, None);
    }
}

#[derive(Debug, Clone)]
pub struct Gauge {
    tracer: Arc<GaugeTracer>,
}

deref_tracer!(Gauge);

impl Gauge {
    /// Increments the value by the specific delta.
    pub fn inc(&self, delta: f64) {
        self.tracer.inc(delta, None);
    }

    /// Decrements the value by the specific delta.
    pub fn dec(&self, delta: f64) {
        self.tracer.dec(delta, None);
    }

    /// Set the value.
    pub fn set(&self, delta: f64) {
        self.tracer.set(delta, None);
    }
}

#[derive(Debug, Clone)]
pub struct Logger {
    tracer: Arc<LogTracer>,
}

deref_tracer!(Logger);

impl Logger {
    /// Writes a message.
    pub fn log(&self, msg: impl ToString) {
        self.tracer.log(msg.to_string(), None);
    }
}
