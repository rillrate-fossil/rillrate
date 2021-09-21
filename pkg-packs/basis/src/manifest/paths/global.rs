use super::tracer::PathsTracer;
use once_cell::sync::Lazy;

pub static PATHS: Lazy<PathsTracer> = Lazy::new(PathsTracer::new);
