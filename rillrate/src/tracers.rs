//! The module with all adapted tracers.

use anyhow::Error;
use rill_engine::tracers::data::{
    CounterTracer, DictTracer, GaugeTracer, HistogramTracer, LoggerTracer, PulseTracer, TableTracer,
};
use rill_protocol::io::provider::Col;
use std::ops::Deref;
use std::sync::Arc;

macro_rules! impl_tracer {
    ($wrapper:ident < $tracer:ident > ( $( $arg:ident : $typ:ty ),* )) => {
        impl Deref for $wrapper {
            type Target = $tracer;

            fn deref(&self) -> &Self::Target {
                self.tracer.deref()
            }
        }

        impl $wrapper {
            /// Creates an instance of the tracer.
            pub fn create(path: &str, $( $arg : $typ ),*) -> Result<Self, Error> {
                let path = path.parse()?;
                let tracer = $tracer::new(path, $( $arg ),*);
                Ok(Self {
                    tracer: Arc::new(tracer),
                })
            }
        }
    };
}

/// `Counter` tracer.
#[derive(Debug, Clone)]
pub struct Counter {
    tracer: Arc<CounterTracer>,
}

impl_tracer!(Counter<CounterTracer>());

impl Counter {
    /// Increments value by the sepcific delta.
    pub fn inc(&self, delta: f64) {
        self.tracer.inc(delta, None);
    }
}

/// `Gauge` tracer.
#[derive(Debug, Clone)]
pub struct Gauge {
    tracer: Arc<GaugeTracer>,
}

impl_tracer!(Gauge<GaugeTracer>(min: f64, max: f64));

impl Gauge {
    /// Increments value by the sepcific delta.
    pub fn set(&self, value: f64) {
        self.tracer.set(value, None);
    }
}

/// `Histogram` tracer.
#[derive(Debug, Clone)]
pub struct Histogram {
    tracer: Arc<HistogramTracer>,
}

impl_tracer!(Histogram<HistogramTracer>(levels: &[f64]));

impl Histogram {
    /// Adds a value to the histogram.
    pub fn add(&self, value: f64) {
        self.tracer.add(value, None);
    }
}

/// `Pulse` tracer.
#[derive(Debug, Clone)]
pub struct Pulse {
    tracer: Arc<PulseTracer>,
}

impl_tracer!(Pulse<PulseTracer>(depth: Option<u32>));

impl Pulse {
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

/// `Logger` tracer.
#[derive(Debug, Clone)]
pub struct Logger {
    tracer: Arc<LoggerTracer>,
}

impl_tracer!(Logger<LoggerTracer>());

impl Logger {
    /// Writes a message.
    pub fn log(&self, msg: impl ToString) {
        self.tracer.log(msg.to_string(), None);
    }
}

/// `Dict` tracer.
#[derive(Debug, Clone)]
pub struct Dict {
    tracer: Arc<DictTracer>,
}

impl_tracer!(Dict<DictTracer>());

impl Dict {
    /// Writes a message.
    pub fn set(&self, key: impl ToString, value: impl ToString) {
        self.tracer.set(key, value, None);
    }
}

/// `Table` tracer.
#[derive(Debug, Clone)]
pub struct Table {
    tracer: Arc<TableTracer>,
}

impl_tracer!(Table<TableTracer>(columns: Vec<(Col, Option<String>)>));

impl Table {}
