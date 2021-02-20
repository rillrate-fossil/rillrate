//! Tracers to send specific types of tracing data.

pub mod tracer;
pub use tracer::Tracer;

pub(crate) mod counter;
pub use counter::CounterTracer;

pub(crate) mod gauge;
pub use gauge::GaugeTracer;

pub(crate) mod logger;
pub use logger::LogTracer;

// TODO: Remove completely
//mod protected;
//use protected::ProtectedTracer;
