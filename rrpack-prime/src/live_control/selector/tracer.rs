use super::state::*;
use crate::auto_path::AutoPath;
use crate::manifest::BindedTracer;
use derive_more::{Deref, DerefMut};
use rill_derive::TracerOpts;
use rill_protocol::flow::core::FlowMode;

#[derive(TracerOpts, Default)]
pub struct SelectorOpts {
    pub label: Option<String>,
    pub options: Vec<String>,
}

impl SelectorOpts {
    /*
    pub fn option(mut self, opt: impl ToString) -> Self {
        self.options.push(opt.to_string());
        self
    }

    pub fn options<T>(mut self, opts: impl IntoIterator<Item = T>) -> Self
    where
        String: From<T>,
    {
        self.options.extend(opts.into_iter().map(String::from));
        self
    }
    */
}

impl From<SelectorOpts> for SelectorSpec {
    fn from(opts: SelectorOpts) -> Self {
        Self {
            label: opts.label.unwrap_or_else(|| "Selector".into()),
            options: opts.options,
        }
    }
}

#[derive(Debug, Deref, DerefMut, Clone)]
pub struct Selector {
    tracer: BindedTracer<SelectorState>,
}

impl Selector {
    pub fn new(auto_path: impl Into<AutoPath>, spec: impl Into<SelectorSpec>) -> Self {
        let tracer = BindedTracer::new(auto_path.into(), FlowMode::Realtime, spec.into());
        Self { tracer }
    }

    pub fn apply(&self, value: Option<String>) {
        let msg = SelectorEvent {
            update_selected: value,
        };
        self.tracer.send(msg, None);
    }
}
