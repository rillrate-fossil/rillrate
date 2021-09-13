use super::{Msg, OnBridgeEvent, Widget, WidgetContext};
use crate::agents::graphics::{GraphicsAgent, GraphicsRequest, GraphicsResponse};
use yew::{Bridge, NodeRef};

pub struct Graphics<'a> {
    link: &'a mut dyn Bridge<GraphicsAgent>,
}

impl<'a> Graphics<'a> {
    pub fn on_frame(&mut self, active: bool) {
        let msg = GraphicsRequest::OnFrame(active);
        self.link.send(msg);
    }

    pub fn track_size(&mut self, node_ref: NodeRef) {
        let msg = GraphicsRequest::TrackSize(node_ref);
        self.link.send(msg);
    }
}

impl<T: Widget> WidgetContext<T> {
    pub fn graphics(&mut self) -> Graphics<'_>
    where
        T: OnBridgeEvent<GraphicsAgent>,
    {
        let link = self.graphics.get_mut_linked(&self.link);
        Graphics { link }
    }
}

impl<T: Widget> From<GraphicsResponse> for Msg<T> {
    fn from(response: GraphicsResponse) -> Self {
        Self::GraphicsIncoming(response)
    }
}
