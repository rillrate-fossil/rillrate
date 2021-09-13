use super::{Context, Msg, Widget, WidgetRuntime};
use anyhow::{anyhow, Error};
use std::collections::HashMap;
use std::hash::Hash;
use std::ops::DerefMut;
use std::rc::Rc;
use yew::worker::Agent;
use yew::{Bridge, Bridged, ComponentLink};

/// This `Hook` technique is used to resolve conflicing inmplementations
/// of `OnWireEvent` and allow to have multiple implementation of that trait.
pub trait AgentHook: 'static {
    type Agent: Agent;
}

impl<T: Agent> AgentHook for T {
    // TODO: Maybe use `Output` here?
    type Agent = Self;
}

pub trait OnWireEvent<H: AgentHook>: Widget {
    fn on_wire(
        &mut self,
        _tag: &Self::Tag,
        _response: <H::Agent as Agent>::Output,
        _ctx: &mut Context<Self>,
    ) -> Result<(), Error> {
        let self_type_name = std::any::type_name::<Self>();
        let type_name = std::any::type_name::<H::Agent>();
        Err(anyhow!(
            "No implementation for incoming event from the agent: {} of {}.",
            type_name,
            self_type_name
        ))
    }
}

pub type WiredHandler<T, E, W> =
    &'static dyn Fn(&mut W, &T, E, &mut Context<W>) -> Result<(), Error>;

pub struct TagRegistry<TAG, ID> {
    // TODO: Use more effective bi-deirctional map here
    tag_to_id: HashMap<Rc<TAG>, ID>,
    id_to_tag: HashMap<ID, Rc<TAG>>,
}

impl<TAG, ID> Default for TagRegistry<TAG, ID> {
    fn default() -> Self {
        Self {
            tag_to_id: HashMap::new(),
            id_to_tag: HashMap::new(),
        }
    }
}

impl<TAG, ID> TagRegistry<TAG, ID>
where
    TAG: Eq + Hash,
    ID: Clone + Eq + Hash,
{
    pub fn insert(&mut self, tag: TAG, id: ID) {
        let tag = Rc::new(tag);
        self.tag_to_id.insert(tag.clone(), id.clone());
        self.id_to_tag.insert(id, tag);
    }

    pub fn remove(&mut self, id: &ID) {
        if let Some(tag) = self.id_to_tag.remove(id) {
            self.tag_to_id.remove(&tag);
        }
    }

    pub fn get(&self, tag: &TAG) -> Option<&ID> {
        self.tag_to_id.get(tag)
    }

    pub fn tag(&self, id: &ID) -> Option<Rc<TAG>> {
        self.id_to_tag.get(id).cloned()
    }

    pub fn all_tags(&self) -> Vec<Rc<TAG>> {
        self.tag_to_id.keys().cloned().collect()
    }

    pub fn is_empty(&self) -> bool {
        self.tag_to_id.is_empty()
    }
}

pub struct WiredBridge<ID, T: Agent, W: Widget> {
    link: Option<Box<dyn Bridge<T>>>,
    // TODO: Join `handler` and `registry` and keep both under single `Option`
    // This filled only if subscribe method called
    handler: Option<WiredHandler<W::Tag, T::Output, W>>,
    registry: TagRegistry<W::Tag, ID>,
}

impl<ID, T: Agent, W: Widget> Default for WiredBridge<ID, T, W> {
    fn default() -> Self {
        Self {
            link: None,
            handler: None,
            registry: TagRegistry::default(),
        }
    }
}

impl<ID, T: Agent, W: Widget> WiredBridge<ID, T, W> {
    pub fn is_linked(&self) -> bool {
        self.link.is_some()
    }

    pub fn handler(&self) -> Option<WiredHandler<W::Tag, T::Output, W>> {
        self.handler.clone()
    }

    pub fn registry(&mut self) -> &mut TagRegistry<W::Tag, ID> {
        &mut self.registry
    }
}

impl<ID, T: Agent, W: Widget> WiredBridge<ID, T, W> {
    pub fn activate_link(&mut self, widget_link: &ComponentLink<WidgetRuntime<W>>)
    where
        Msg<W>: From<T::Output>,
    {
        if self.link.is_none() {
            let callback = widget_link.callback(Msg::from);
            let link = T::bridge(callback);
            self.link = Some(link);
        }
    }

    pub fn activate_handler<H>(&mut self)
    where
        H: AgentHook<Agent = T>,
        W: OnWireEvent<H>,
    {
        if self.handler.is_none() {
            let handler = &<W as OnWireEvent<H>>::on_wire;
            self.handler = Some(handler);
        }
    }

    pub fn get_mut_linked<H>(
        &mut self,
        widget_link: &ComponentLink<WidgetRuntime<W>>,
    ) -> (&mut dyn Bridge<T>, &mut TagRegistry<W::Tag, ID>)
    where
        H: AgentHook<Agent = T>,
        Msg<W>: From<T::Output>,
        W: OnWireEvent<H>,
    {
        self.activate_link(widget_link);
        self.activate_handler();
        let bridge = self.link.as_mut().map(Box::deref_mut).unwrap();
        (bridge, &mut self.registry)
    }
}
