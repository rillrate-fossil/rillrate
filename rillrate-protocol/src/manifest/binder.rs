use crate::manifest::descriptions_list::DescriptionsListTracer;
use crate::manifest::groups_list::{GroupsListTracer, GroupsListUpdate};
use derive_more::{Deref, DerefMut};
use once_cell::sync::Lazy;
use rill_engine::tracers::tracer::Tracer;
use rill_protocol::flow::core::Flow;
use rill_protocol::io::provider::{Description, Path};
use std::collections::HashSet;
use std::ops::Deref;
use std::sync::{Arc, Mutex};

static DESCRIPTIONS: Lazy<DescriptionsListTracer> = Lazy::new(DescriptionsListTracer::new);
static GROUPS: Lazy<GroupsListTracer> = Lazy::new(GroupsListTracer::new);

/// `Binded` wraps a tracer to automatically track it in the global `DescriptionFlow`.
#[derive(Deref, DerefMut, Debug, Clone)]
pub struct Binded<T> {
    #[deref]
    #[deref_mut]
    tracer: T,
    description: Arc<DescriptionBinder>,
    group: Arc<Mutex<GroupBinder>>,
}

impl<T> Binded<T> {
    pub fn new<F>(tracer: T) -> Self
    where
        F: Flow,
        T: Deref<Target = Tracer<F>>,
    {
        let desc = tracer.description();
        let description = DescriptionBinder::new(desc);
        let group = GroupBinder::new(desc);
        Self {
            tracer,
            description: Arc::new(description),
            group: Arc::new(Mutex::new(group)),
        }
    }

    pub fn join(&self, group: Path) {
        if let Ok(mut groups) = self.group.lock() {
            groups.insert(group);
        } else {
            log::error!("Can't lock the groups binder to join the group {}", group);
        }
    }

    pub fn leave(&self, group: Path) {
        if let Ok(mut groups) = self.group.lock() {
            groups.remove(group);
        } else {
            log::error!("Can't lock the groups binder to leave the group {}", group);
        }
    }
}

#[derive(Debug)]
struct DescriptionBinder {
    path: Path,
}

impl DescriptionBinder {
    fn new(description: &Description) -> Self {
        let path = description.path.clone();
        DESCRIPTIONS.add_record(path.clone(), description.clone());
        Self { path }
    }
}

impl Drop for DescriptionBinder {
    fn drop(&mut self) {
        DESCRIPTIONS.remove_record(self.path.clone());
    }
}

#[derive(Debug)]
struct GroupBinder {
    path: Path,
    groups: HashSet<Path>,
}

impl GroupBinder {
    fn new(description: &Description) -> Self {
        let path = description.path.clone();
        Self {
            path,
            groups: HashSet::new(),
        }
    }

    fn insert(&mut self, group: Path) {
        self.groups.insert(group.clone());
        let update = GroupsListUpdate::JoinGroup { path: group };
        GROUPS.update_record(self.path.clone(), update);
    }

    fn remove(&mut self, group: Path) {
        self.groups.remove(&group);
        let update = GroupsListUpdate::LeaveGroup { path: group };
        GROUPS.update_record(self.path.clone(), update);
    }
}

impl Drop for GroupBinder {
    fn drop(&mut self) {
        for group in self.groups.drain() {
            let update = GroupsListUpdate::LeaveGroup { path: group };
            GROUPS.update_record(self.path.clone(), update);
        }
    }
}
