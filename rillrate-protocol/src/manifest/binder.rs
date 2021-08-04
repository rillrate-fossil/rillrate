use crate::manifest::descriptions_list::DescriptionsListTracer;
use derive_more::{Deref, DerefMut};
use once_cell::sync::Lazy;
use rill_engine::tracers::tracer::Tracer;
use rill_protocol::flow::core::Flow;
use rill_protocol::io::provider::{Description, Path};
use std::ops::Deref;
use std::sync::Arc;

/// `Binded` wraps a tracer to automatically track it in the global `DescriptionFlow`.
#[derive(Deref, DerefMut, Debug, Clone)]
pub struct Binded<T> {
    #[deref]
    #[deref_mut]
    tracer: T,
    _binder: Arc<DescriptionBinder>,
}

impl<T> Binded<T> {
    pub fn new<F>(tracer: T) -> Self
    where
        F: Flow,
        T: Deref<Target = Tracer<F>>,
    {
        let description = tracer.description().clone();
        let binder = DescriptionBinder::new(description);
        Self {
            tracer,
            _binder: Arc::new(binder),
        }
    }
}

static DESCRIPTIONS: Lazy<DescriptionsListTracer> = Lazy::new(DescriptionsListTracer::new);

#[derive(Debug)]
struct DescriptionBinder {
    path: Path,
}

impl DescriptionBinder {
    fn new(description: Description) -> Self {
        let path = description.path.clone();
        DESCRIPTIONS.add_record(path.clone(), description);
        Self { path }
    }
}

impl Drop for DescriptionBinder {
    fn drop(&mut self) {
        DESCRIPTIONS.remove_record(self.path.clone());
    }
}
