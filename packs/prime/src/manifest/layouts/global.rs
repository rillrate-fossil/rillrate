use super::tracer::LayoutsTracer;
use once_cell::sync::Lazy;

pub static LAYOUTS: Lazy<LayoutsTracer> = Lazy::new(LayoutsTracer::new);
