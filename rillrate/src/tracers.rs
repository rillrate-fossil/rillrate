use anyhow::Error;
use rill::tracers::{CounterTracer, GaugeTracer, LogTracer};
use std::ops::Deref;
use std::sync::Arc;

macro_rules! impl_tracer {
    ($wrapper:ident < $tracer:ident >) => {
        impl Deref for $wrapper {
            type Target = $tracer;

            fn deref(&self) -> &Self::Target {
                self.tracer.deref()
            }
        }

        impl $wrapper {
            pub fn create(path: &str) -> Result<Self, Error> {
                let path = path.parse()?;
                let tracer = $tracer::new(path);
                Ok(Self {
                    tracer: Arc::new(tracer),
                })
            }
        }
    };
}

#[derive(Debug, Clone)]
pub struct Counter {
    tracer: Arc<CounterTracer>,
}

impl_tracer!(Counter<CounterTracer>);

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

impl_tracer!(Gauge<GaugeTracer>);

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

impl_tracer!(Logger<LogTracer>);

impl Logger {
    /// Writes a message.
    pub fn log(&self, msg: impl ToString) {
        self.tracer.log(msg.to_string(), None);
    }
}
