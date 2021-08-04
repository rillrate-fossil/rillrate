use crate::manifest::descriptions_list::DescriptionsListTracer;
use crate::manifest::groups_list::{GroupsListTracer, GroupsListUpdate};
use derive_more::{Deref, DerefMut};
use once_cell::sync::Lazy;
use rill_engine::tracers::tracer::Tracer;
use rill_protocol::flow::core::Flow;
use rill_protocol::io::provider::{Description, EntryId};
use std::ops::Deref;

static DESCRIPTIONS: Lazy<DescriptionsListTracer> = Lazy::new(DescriptionsListTracer::new);
static GROUPS: Lazy<GroupsListTracer> = Lazy::new(GroupsListTracer::new);

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

    fn pair(&self) -> (EntryId, EntryId) {
        let mut path = self.description.path.clone().into_iter();
        assert!(path.len() == 2);
        let name = path.next().unwrap();
        let group = path.next().unwrap();
        (group, name)
    }

    fn register(&self) {
        let (group, name) = self.pair();
        DESCRIPTIONS.add_record(name.clone(), self.description.clone());
        let update = GroupsListUpdate::JoinGroup { entry_id: name };
        GROUPS.update_record(group, update);
    }

    fn unregister(&self) {
        let (group, name) = self.pair();
        DESCRIPTIONS.remove_record(name.clone());
        let update = GroupsListUpdate::LeaveGroup { entry_id: name };
        GROUPS.update_record(group, update);
    }
}

impl<T> Drop for Binded<T> {
    fn drop(&mut self) {
        self.unregister();
    }
}
