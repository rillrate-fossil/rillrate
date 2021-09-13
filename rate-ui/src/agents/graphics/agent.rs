use super::reframer::Reframer;
use derive_more::{From, Into};
use std::collections::{HashMap, HashSet};
use std::time::Duration;
use typed_slab::TypedSlab;
use web_sys::Element;
use yew::services::interval::{IntervalService, IntervalTask};
use yew::services::resize::{ResizeService, ResizeTask};
use yew::worker::{Agent, AgentLink, Context, HandlerId};
use yew::NodeRef;

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct RectSize {
    pub width: usize,
    pub height: usize,
}

#[derive(Debug, Clone, Copy, From, Into, PartialEq, Eq, Hash)]
struct NodeTrackId(usize);

struct NodeTracker {
    who: HandlerId,
    node_ref: NodeRef,
    size: RectSize,
}

impl NodeTracker {
    fn new(who: HandlerId, node_ref: NodeRef) -> Self {
        Self {
            who,
            node_ref,
            size: RectSize::default(),
        }
    }

    fn changed(&mut self) -> Option<RectSize> {
        if let Some(element) = self.node_ref.cast::<Element>() {
            let rect = element.get_bounding_client_rect();
            let real_size = RectSize {
                width: rect.width() as usize,
                height: rect.height() as usize,
            };
            if self.size != real_size {
                self.size = real_size;
                return Some(self.size.clone());
            }
        }
        None
    }
}

pub struct GraphicsAgent {
    link: AgentLink<Self>,
    reframer: Reframer,
    // TODO: Is it safe for the task to remove this service here?
    //resizer: ResizeService,
    #[allow(dead_code)]
    resize_task: ResizeTask,
    //on_resize: HashSet<HandlerId>,
    on_frame: HashSet<HandlerId>,

    #[allow(dead_code)]
    check_sizes: IntervalTask,
    tracking_nodes: TypedSlab<NodeTrackId, NodeTracker>,
    assigned_ids: HashMap<HandlerId, HashSet<NodeTrackId>>,
}

pub enum Msg {
    RenderFrame(f64),
    Resized,
    CheckSizes,
}

#[derive(Debug)]
pub enum GraphicsRequest {
    //OnResize(bool),
    OnFrame(bool),
    TrackSize(NodeRef),
}

#[derive(Debug)]
pub enum GraphicsResponse {
    //Resized,
    Frame,
    SizeChanged(RectSize),
}

impl Agent for GraphicsAgent {
    type Reach = Context<Self>;
    type Message = Msg;
    type Input = GraphicsRequest;
    type Output = GraphicsResponse;

    fn create(link: AgentLink<Self>) -> Self {
        let callback = link.callback(Msg::RenderFrame);
        let reframer = Reframer::new(callback);
        let callback = link.callback(|_| Msg::Resized);
        let resize_task = ResizeService::register(callback);
        let callback = link.callback(|_| Msg::CheckSizes);
        let check_sizes = IntervalService::spawn(Duration::from_millis(400), callback);
        Self {
            link,
            reframer,
            //resizer,
            resize_task,
            //on_resize: HashSet::new(),
            on_frame: HashSet::new(),

            check_sizes,
            tracking_nodes: TypedSlab::new(),
            assigned_ids: HashMap::new(),
        }
    }

    fn update(&mut self, msg: Self::Message) {
        match msg {
            Msg::RenderFrame(_) => {
                for who in self.on_frame.iter() {
                    self.link.respond(*who, GraphicsResponse::Frame);
                }
                self.reframer.request_next_frame();
            }
            /*
            Msg::Resized => {
                for who in self.on_resize.iter() {
                    self.link.respond(*who, GraphicsResponse::Resized);
                }
            }
            */
            Msg::Resized | Msg::CheckSizes => {
                // Checking this on resize, because if window size changes
                // doesn't mean the component size changed as well.
                for (_, tracker) in self.tracking_nodes.iter_mut() {
                    if let Some(size) = tracker.changed() {
                        self.link
                            .respond(tracker.who, GraphicsResponse::SizeChanged(size));
                    }
                }
            }
        }
    }

    fn handle_input(&mut self, request: Self::Input, who: HandlerId) {
        match request {
            GraphicsRequest::TrackSize(node_ref) => {
                let mut tracker = NodeTracker::new(who, node_ref);
                // Send the size and relayout immediately
                if let Some(size) = tracker.changed() {
                    self.link
                        .respond(tracker.who, GraphicsResponse::SizeChanged(size));
                }
                // Keep the node for size tracking
                let track_id = self.tracking_nodes.insert(tracker);
                self.assigned_ids.entry(who).or_default().insert(track_id);
            }
            /*
            GraphicsRequest::OnResize(active) => {
                if active {
                    self.on_resize.insert(who);
                    if self.on_resize.len() == 1 {
                        let callback = self.link.callback(|_| Msg::Resized);
                        let resize_task = self.resizer.register(callback);
                        self.resize_task = Some(resize_task);
                    }
                    self.link.respond(who, GraphicsResponse::Resized);
                } else {
                    self.on_resize.remove(&who);
                    if self.on_resize.is_empty() {
                        self.resize_task.take();
                    }
                }
            }
            */
            GraphicsRequest::OnFrame(active) => {
                if active {
                    self.on_frame.insert(who);
                    if self.on_frame.len() == 1 {
                        self.reframer.request_next_frame();
                    }
                    self.link.respond(who, GraphicsResponse::Frame);
                } else {
                    self.on_frame.remove(&who);
                    if self.on_frame.is_empty() {
                        self.reframer.interrupt();
                    }
                }
            }
        }
    }

    fn connected(&mut self, _id: HandlerId) {
        //log::trace!("Connected to Graphics: {:?}", id);
    }

    fn disconnected(&mut self, id: HandlerId) {
        //log::trace!("Disconnected from Graphics: {:?}", id);
        self.handle_input(GraphicsRequest::OnFrame(false), id);
        //self.handle_input(GraphicsRequest::OnResize(false), id);
        if let Some(track_ids) = self.assigned_ids.remove(&id) {
            for track_id in track_ids {
                self.tracking_nodes.remove(track_id);
            }
        }
    }
}
