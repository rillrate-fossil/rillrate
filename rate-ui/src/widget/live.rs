use super::{DropHook, Msg, OnWireEvent, Widget, WidgetContext};
use crate::agents::live::registry::REGISTRY;
use crate::agents::live::wire::{WireEnvelope, WireTask};
use crate::agents::live::{LiveAgent, LiveRequest, LiveResponse};
use crate::widget::wired_bridge::{AgentHook, TagRegistry};
use rill_protocol::io::client::ClientReqId;
use yew::Bridge;

pub struct Live<'a, T: Widget> {
    link: &'a mut dyn Bridge<LiveAgent>,
    registry: &'a mut TagRegistry<T::Tag, ClientReqId>,
}

impl<'a, T: Widget> Live<'a, T> {
    pub fn wire(&mut self, tag: T::Tag, task: impl WireTask) {
        let action = Box::new(task);
        let req_id = REGISTRY.acquire();
        self.registry.insert(tag, req_id);
        let request = LiveRequest::Wire(action);
        let envelope = WireEnvelope::new(req_id, request);
        self.link.send(envelope);
    }

    pub fn unwire(&mut self, tag: &T::Tag) {
        if let Some(req_id) = self.registry.get(tag) {
            let request = LiveRequest::TerminateWire;
            let envelope = WireEnvelope::new(*req_id, request);
            self.link.send(envelope);
        } else {
        }
    }
}

impl<T: Widget> WidgetContext<T> {
    pub fn live<H>(&mut self) -> Live<'_, T>
    where
        H: AgentHook<Agent = LiveAgent>,
        T: OnWireEvent<H>,
    {
        if !self.live.is_linked() {
            // TODO: Add DropHook
            let hook: DropHook<T> = Box::new(|_widget, this| {
                let (link, registry) = this.live.get_mut_linked(&this.link);
                if !registry.is_empty() {
                    let all_tags = registry.all_tags();
                    let mut live: Live<'_, T> = Live { link, registry };
                    for tag in all_tags {
                        live.unwire(tag.as_ref());
                    }
                }
            });
            self.drop_hooks.push(hook);
        }
        let (link, registry) = self.live.get_mut_linked(&self.link);
        Live { link, registry }
    }
}

impl<T: Widget> From<WireEnvelope<ClientReqId, LiveResponse>> for Msg<T> {
    fn from(response: WireEnvelope<ClientReqId, LiveResponse>) -> Self {
        Self::LiveIncomingWired(response)
    }
}
