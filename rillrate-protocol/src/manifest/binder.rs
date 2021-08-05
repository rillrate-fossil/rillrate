use crate::manifest::descriptions_list::DescriptionsListTracer;
use derive_more::{Deref, DerefMut};
use once_cell::sync::Lazy;
use rill_engine::tracers::tracer::Tracer;
use rill_protocol::flow::core::Flow;
use rill_protocol::io::provider::Description;
use std::ops::Deref;

static DESCRIPTIONS: Lazy<DescriptionsListTracer> = Lazy::new(DescriptionsListTracer::new);

/// `Binded` wraps a tracer to automatically track it in the global `DescriptionFlow`.
#[derive(Deref, DerefMut, Debug, Clone)]
pub struct Binded<T> {
    #[deref]
    #[deref_mut]
    tracer: T,
    description: Description,
}

impl<T> Binded<T> {
    pub fn new<F>(tracer: T) -> Self
    where
        F: Flow,
        T: Deref<Target = Tracer<F>>,
    {
        let description = tracer.description().clone();
        let this = Self {
            tracer,
            description,
        };
        this.register();
        this
    }

    fn register(&self) {
        let path = self.description.path.clone();
        DESCRIPTIONS.add_record(path, self.description.clone());
    }

    fn unregister(&self) {
        let path = self.description.path.clone();
        DESCRIPTIONS.remove_record(path.clone());
    }
}

impl<T> Drop for Binded<T> {
    fn drop(&mut self) {
        self.unregister();
    }
}
