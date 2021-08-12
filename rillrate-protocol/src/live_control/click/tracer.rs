use super::state::*;
use crate::base_control::emit_control::EmitControlTracer;
use crate::manifest::Binded;

pub struct Click {
    _tracer: Binded<EmitControlTracer<ClickSpec>>,
}
