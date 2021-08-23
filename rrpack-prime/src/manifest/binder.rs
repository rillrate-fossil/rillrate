use crate::auto_path::AutoPath;
use crate::manifest::description::{PackFlow, PackFlowDescription};
use crate::manifest::descriptions_list::DescriptionsListTracer;
use derive_more::{Deref, DerefMut};
use once_cell::sync::Lazy;
use rill_engine::tracers::tracer::Tracer;
use rill_protocol::flow::core::FlowMode;
use std::sync::Arc;

static DESCRIPTIONS: Lazy<DescriptionsListTracer> = Lazy::new(DescriptionsListTracer::new);

/// `Binded` wraps a tracer to automatically track it in the global `DescriptionFlow`.
#[derive(Deref, DerefMut, Debug, Clone)]
pub struct BindedTracer<T: PackFlow> {
    #[deref]
    #[deref_mut]
    tracer: Tracer<T>,
    binder: Binder,
}

impl<T: PackFlow> BindedTracer<T> {
    pub fn new<S>(auto_path: AutoPath, mode: FlowMode, spec: S) -> Self
    where
        S: Into<T>,
    {
        let state = spec.into();
        let tracer = Tracer::new(state, auto_path.into(), mode);
        let binder = Binder::new(&tracer);
        Self { tracer, binder }
    }
}

/// `Binder` wraps a tracer to automatically track it in the global `DescriptionFlow`.
#[derive(Deref, DerefMut, Debug, Clone)]
struct Binder {
    /// Wrapped with `Arc` to have a single instance of inner only.
    inner: Arc<BinderInner>,
}

impl Binder {
    pub fn new<T: PackFlow>(tracer: &Tracer<T>) -> Self {
        let desc = tracer.description();
        let description = PackFlowDescription {
            path: desc.path.clone(),
            layer: T::layer(),
            stream_type: desc.stream_type.clone(),
        };
        let inner = BinderInner { description };
        let this = Self {
            inner: Arc::new(inner),
        };
        this.inner.register();
        this
    }
}

#[derive(Deref, DerefMut, Debug, Clone)]
struct BinderInner {
    description: PackFlowDescription,
}

impl BinderInner {
    fn register(&self) {
        let path = self.description.path.clone();
        //log::debug!("REGISTERING: {}", path);
        DESCRIPTIONS.add_path(path, self.description.clone());
    }

    fn unregister(&self) {
        let path = self.description.path.clone();
        //log::debug!("UNREGISTERING: {}", path);
        DESCRIPTIONS.remove_path(path);
    }
}

impl Drop for BinderInner {
    fn drop(&mut self) {
        self.unregister();
    }
}
