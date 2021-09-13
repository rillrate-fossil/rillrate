use crate::agents::live::wire::{DoAction, Subscription, WireEnvelope};
use crate::agents::live::{LiveAgent, LiveResponse};
//use crate::common;
use crate::widget::wired_bridge::AgentHook;
use crate::widget::{Context, OnWireEvent, Widget};
use anyhow::Error;
use rill_protocol::diff::diff;
use rill_protocol::flow::core::Flow;
use rill_protocol::io::client::{ClientReqId, ClientResponse};
use rill_protocol::io::provider::Path;
use std::collections::{HashMap, HashSet};
use yew::{Callback, Properties};

pub trait WiredWidget<M>: Widget<Tag = Option<Path>, Meta = M> {
    type Flow: Flow;

    fn state_changed(&mut self, _reloaded: bool, _ctx: &mut Context<Self>) {}

    fn state_update(
        &mut self,
        _tag: &Path,
        _event: &<Self::Flow as Flow>::Event,
        _reloaded: &mut bool,
        _ctx: &mut Context<Self>,
    ) {
    }
}

#[derive(Properties, Clone, PartialEq)]
pub struct SingleFlowProps {
    pub path: Path,
    #[prop_or_default]
    pub triggered: Option<Callback<()>>,
}

impl SingleFlowProps {
    pub fn notify(&self) {
        if let Some(callback) = self.triggered.as_ref() {
            callback.emit(());
        }
    }
}

pub struct SingleFlowMeta<T: WiredWidget<Self>> {
    state: Option<T::Flow>,
    wire: Option<Path>,
}

impl<T> Default for SingleFlowMeta<T>
where
    T: WiredWidget<Self>,
{
    fn default() -> Self {
        Self {
            state: None,
            wire: None,
        }
    }
}

impl<T> SingleFlowMeta<T>
where
    T: WiredWidget<Self>,
{
    pub fn state(&self) -> Option<&T::Flow> {
        self.state.as_ref()
    }

    /*
    pub fn active(&self) -> Option<&Path> {
        self.wire.as_ref()
    }
    */
}

impl<T> Context<T>
where
    T: WiredWidget<SingleFlowMeta<T>>,
{
    pub fn do_action(&mut self, action: <T::Flow as Flow>::Action) {
        if let Some(path) = self.meta().wire.clone() {
            let do_action = DoAction::<T::Flow>::new(path, action);
            self.live().wire(None, do_action);
            // TODO: Redraw is not needed here. Remove.
            //self.redraw();
        } else {
            log::error!("No path to do action: {:?}", action);
        }
    }

    /// SingleFlow mode also compatible with any other actions.
    pub fn do_action_of<F: Flow>(&mut self, path: Path, action: F::Action) {
        let do_action = DoAction::<F>::new(path, action);
        self.live().wire(None, do_action);
    }

    /// It unwires automatically.
    pub fn rewire(&mut self, path: Path) {
        // Don't rewire if path the same
        if self.meta().wire.as_ref() != Some(&path) {
            self.unwire();
            self.meta_mut().state.take();
            let wire_task = Subscription::new(path.clone());
            let new_wire = Some(path);
            self.meta_mut().wire = new_wire.clone();
            self.live().wire(new_wire, wire_task);
            self.redraw();
        }
    }

    pub fn unwire(&mut self) {
        if let Some(path) = self.meta_mut().wire.take() {
            self.live().unwire(&Some(path));
        }
    }
}

/*
pub fn loading_view() -> Html {
    html! {
        <common::Spinner />
    }
}
*/

pub struct SingleHook;

impl AgentHook for SingleHook {
    type Agent = LiveAgent;
}

impl<T> OnWireEvent<SingleHook> for T
where
    T: WiredWidget<SingleFlowMeta<Self>>,
{
    fn on_wire(
        &mut self,
        tag: &Self::Tag,
        event: WireEnvelope<ClientReqId, LiveResponse>,
        ctx: &mut Context<Self>,
    ) -> Result<(), Error> {
        match (tag.as_ref(), event.data) {
            (Some(path), event) => match event {
                LiveResponse::Forwarded(response) => {
                    let mut reloaded = false;
                    match response {
                        ClientResponse::State(data) => {
                            let res = T::Flow::unpack_state(&data);
                            match res {
                                Ok(state) => {
                                    ctx.meta_mut().state = Some(state);
                                    reloaded = true;
                                }
                                Err(err) => {
                                    log::error!("Can't unpack the state: {}", err);
                                }
                            }
                        }
                        ClientResponse::Delta(data) => {
                            let res = T::Flow::unpack_event(&data);
                            match res {
                                Ok(event) => {
                                    self.state_update(path, &event, &mut reloaded, ctx);
                                    if let Some(state) = ctx.meta_mut().state.as_mut() {
                                        state.apply(event);
                                    }
                                }
                                Err(err) => {
                                    log::error!("Can't unpack the delta: {}", err);
                                }
                            }
                        }
                        ClientResponse::Done => {
                            // TODO: What to do when the stream is finished completely?
                        }
                        other => {
                            log::error!("Unexpected message for the single flow: {:?}", other);
                        }
                    }
                    self.state_changed(reloaded, ctx);
                }
                _ => {}
            },
            (None, _) => {
                // Redraw on action
                ctx.redraw();
            }
        }
        Ok(())
    }
}

#[derive(Properties, Clone, PartialEq)]
pub struct MultiFlowProps {
    pub paths: HashSet<Path>,
}

pub struct MultiFlowMeta<T: WiredWidget<Self>> {
    states: HashMap<Path, T::Flow>,
    wires: HashSet<Path>,
}

impl<T> Default for MultiFlowMeta<T>
where
    T: WiredWidget<Self>,
{
    fn default() -> Self {
        Self {
            states: HashMap::new(),
            wires: HashSet::new(),
        }
    }
}

impl<T> MultiFlowMeta<T>
where
    T: WiredWidget<Self>,
{
    pub fn states(&self) -> &HashMap<Path, T::Flow> {
        &self.states
    }
}

impl<T> Context<T>
where
    T: WiredWidget<MultiFlowMeta<T>>,
{
    pub fn rewire_many(&mut self, paths: &HashSet<Path>) {
        let (to_add, to_remove) = diff(&self.meta().wires, paths);
        for path in to_add {
            let wire_task = Subscription::new(path.clone());
            self.live().wire(Some(path.clone()), wire_task);
            self.meta_mut().wires.insert(path);
            // `State/Flow` will be received using `wire`
            self.redraw();
        }
        for path in to_remove {
            self.meta_mut().wires.remove(&path);
            self.meta_mut().states.remove(&path);
            self.live().unwire(&Some(path));
            self.redraw();
        }
    }
}

pub struct MultiHook;

impl AgentHook for MultiHook {
    type Agent = LiveAgent;
}

impl<T> OnWireEvent<MultiHook> for T
where
    T: WiredWidget<MultiFlowMeta<Self>>,
{
    fn on_wire(
        &mut self,
        tag: &Self::Tag,
        event: WireEnvelope<ClientReqId, LiveResponse>,
        ctx: &mut Context<Self>,
    ) -> Result<(), Error> {
        match (tag.as_ref(), event.data) {
            (Some(path), event) => match event {
                LiveResponse::Forwarded(response) => {
                    let mut reloaded = false;
                    match response {
                        ClientResponse::State(data) => {
                            let state = T::Flow::unpack_state(&data).unwrap();
                            ctx.meta_mut().states.insert(path.clone(), state);
                            reloaded = true;
                        }
                        ClientResponse::Delta(data) => {
                            // TODO: Don't `unwrap` here
                            let event = T::Flow::unpack_event(&data).unwrap();
                            self.state_update(path, &event, &mut reloaded, ctx);
                            if let Some(state) = ctx.meta_mut().states.get_mut(path) {
                                state.apply(event);
                            }
                        }
                        ClientResponse::Done => {
                            // TODO: What to do when the stream is finished completely?
                        }
                        other => {
                            log::error!("Unexpected message for the multi flow: {:?}", other);
                        }
                    }
                    self.state_changed(reloaded, ctx);
                }
                _ => {}
            },
            (None, _) => {
                // Redraw on action
                ctx.redraw();
            }
        }
        Ok(())
    }
}
