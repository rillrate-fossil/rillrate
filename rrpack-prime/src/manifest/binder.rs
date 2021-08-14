use crate::manifest::descriptions_list::DescriptionsListTracer;
use derive_more::{Deref, DerefMut};
use once_cell::sync::Lazy;
use rill_engine::tracers::tracer::Tracer;
use rill_protocol::flow::core::Flow;
use rill_protocol::io::provider::Description;
use std::ops::Deref;
use std::sync::Arc;

static DESCRIPTIONS: Lazy<DescriptionsListTracer> = Lazy::new(DescriptionsListTracer::new);

// TODO: Remove and replace with `Binder`! It unregistered wrong!
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
        DESCRIPTIONS.remove_record(path);
    }
}

impl<T> Drop for Binded<T> {
    fn drop(&mut self) {
        self.unregister();
    }
}

/// `Binder` wraps a tracer to automatically track it in the global `DescriptionFlow`.
#[derive(Deref, DerefMut, Debug, Clone)]
pub struct Binder {
    /// Wrapped with `Arc` to have a single instance of inner only.
    inner: Arc<BinderInner>,
}

impl Binder {
    pub fn new<T: Flow>(tracer: &Tracer<T>) -> Self {
        let description = tracer.description().clone();
        let inner = BinderInner { description };
        let this = Self {
            inner: Arc::new(inner),
        };
        this.inner.register();
        this
    }
}

#[derive(Deref, DerefMut, Debug, Clone)]
pub struct BinderInner {
    description: Description,
}

impl BinderInner {
    fn register(&self) {
        let path = self.description.path.clone();
        //log::debug!("REGISTERING: {}", path);
        DESCRIPTIONS.add_record(path, self.description.clone());
    }

    fn unregister(&self) {
        let path = self.description.path.clone();
        //log::debug!("UNREGISTERING: {}", path);
        DESCRIPTIONS.remove_record(path);
    }
}

impl Drop for BinderInner {
    fn drop(&mut self) {
        self.unregister();
    }
}
