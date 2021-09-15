mod graphics;
mod lazy_bridge;
mod live;
mod wired_bridge;
pub mod wired_widget;

use crate::agents::graphics::{GraphicsAgent, GraphicsResponse};
use crate::agents::live::wire::WireEnvelope;
use crate::agents::live::{LiveAgent, LiveResponse};
use anyhow::Error;
use lazy_bridge::LazyBridge;
pub use lazy_bridge::OnBridgeEvent;
use rill_protocol::io::client::ClientReqId;
use std::hash::Hash;
use std::time::Duration;
pub use wired_bridge::OnWireEvent;
use wired_bridge::WiredBridge;
use yew::services::timeout::{TimeoutService, TimeoutTask};
use yew::{Callback, Component, ComponentLink, Html, Properties, ShouldRender};

pub trait Widget: Default + 'static {
    type Event;
    // TODO: Don't `Clone` since the reference required
    type Tag: Clone + Eq + Hash;
    type Properties: Properties + PartialEq;
    type Meta: Default;

    fn init(&mut self, _ctx: &mut Context<Self>) {}

    fn on_props(&mut self, _ctx: &mut Context<Self>) {}

    fn on_event(&mut self, _event: Self::Event, _ctx: &mut Context<Self>) {}

    fn view(&self, ctx: &Context<Self>) -> Html;

    // TODO: Replate to the trait `OnRendered`
    fn rendered(&mut self, _first: bool) -> Result<(), Error> {
        Ok(())
    }
}

pub type Context<T> = WidgetContext<T>;

pub struct WidgetContext<T: Widget> {
    props: T::Properties,
    link: ComponentLink<WidgetRuntime<T>>,

    live: WiredBridge<ClientReqId, LiveAgent, T>,
    graphics: LazyBridge<GraphicsAgent, T>,
    should_render: bool,
    rendered: bool,
    scheduled: Option<TimeoutTask>,

    drop_hooks: Vec<DropHook<T>>,

    // TODO: Store router state here
    // keep them together with `Meta`
    // provide access to it and store in the
    meta: T::Meta,
}

pub type DropHook<T> = Box<dyn FnOnce(&mut T, &mut Context<T>)>;

impl<T: Widget> Drop for WidgetRuntime<T> {
    fn drop(&mut self) {
        if !self.context.drop_hooks.is_empty() {
            let hooks: Vec<_> = self.context.drop_hooks.drain(..).collect();
            for hook in hooks {
                hook(&mut self.widget, &mut self.context);
            }
        }
        /*
        for (req_id, _tag) in self.wires_from_live.drain() {
            let req = LiveRequest::TerminateWire;
            let envelope = WireEnvelope::new(req_id, req);
            self.context.connection.send(envelope);
        }
        */
    }
}

impl<T: Widget> WidgetContext<T> {
    /// Schedule a timeout.
    ///
    /// Note: It's impossible to move to a separate module,
    /// because it requires a link to a `Component`.
    pub fn schedule(&mut self, ms: u64, msg: T::Event) {
        let dur = Duration::from_millis(ms);
        let generator = move |_| Msg::Event(msg);
        let callback = self.link.callback_once(generator);
        let task = TimeoutService::spawn(dur, callback);
        self.scheduled = Some(task);
    }

    pub fn is_scheduled(&self) -> bool {
        self.scheduled.is_some()
    }

    pub fn unschedule(&mut self) {
        self.scheduled.take();
    }
}

impl<T: Widget> WidgetContext<T> {
    pub fn properties(&self) -> &T::Properties {
        &self.props
    }

    pub fn meta(&self) -> &T::Meta {
        &self.meta
    }

    pub fn meta_mut(&mut self) -> &mut T::Meta {
        &mut self.meta
    }

    // TODO: Rename to `schedule_redraw`
    pub fn redraw(&mut self) {
        self.should_render = true;
    }

    pub fn is_rendered(&self) -> bool {
        self.rendered
    }

    pub fn callback<F, IN>(&self, f: F) -> Callback<IN>
    where
        F: Fn(IN) -> T::Event + 'static,
    {
        let generator = move |event| Msg::Event(f(event));
        self.link.callback(generator)
    }

    pub fn event<IN>(&self, msg: impl Into<T::Event>) -> Callback<IN>
    where
        T::Event: Clone,
    {
        let msg = msg.into();
        let generator = move |_| Msg::Event(msg.clone());
        self.link.callback(generator)
    }

    pub fn notification<IN>(&self) -> Callback<IN>
    where
        T: NotificationHandler<IN>,
        IN: 'static,
    {
        let generator = move |event| {
            let holder = NotificationImpl { event: Some(event) };
            Msg::InPlace(Box::new(holder))
        };
        self.link.callback(generator)
    }

    /* not necessary right now
    pub fn send(&mut self, event: T::Event) {
        self.link.send_message(Msg::Event(event));
    }
    */
}

pub trait NotificationHandler<IN>: Widget {
    fn handle(&mut self, event: IN, context: &mut Context<Self>) -> Result<(), Error>;
}

struct NotificationImpl<IN> {
    event: Option<IN>,
}

impl<T, IN> WidgetCallbackFn<T> for NotificationImpl<IN>
where
    T: NotificationHandler<IN> + Widget,
{
    fn handle(&mut self, widget: &mut T, context: &mut Context<T>) -> Result<(), Error> {
        if let Some(event) = self.event.take() {
            widget.handle(event, context)?;
        }
        Ok(())
    }
}

pub trait WidgetCallbackFn<T: Widget> {
    fn handle(&mut self, widget: &mut T, context: &mut Context<T>) -> Result<(), Error>;
}

pub enum Msg<T: Widget> {
    // TODO: Implement handlers as traits. Envelope-based.
    LiveIncomingWired(WireEnvelope<ClientReqId, LiveResponse>),
    GraphicsIncoming(GraphicsResponse),
    Event(T::Event),
    InPlace(Box<dyn WidgetCallbackFn<T>>),
}

pub struct WidgetRuntime<T: Widget> {
    widget: T,
    context: WidgetContext<T>,
}

impl<T: Widget> Component for WidgetRuntime<T> {
    type Message = Msg<T>;
    type Properties = T::Properties;

    fn create(props: Self::Properties, link: ComponentLink<Self>) -> Self {
        let mut context = WidgetContext {
            props,
            link,

            live: WiredBridge::default(),
            graphics: LazyBridge::default(),
            //router: LazyBridge::default(),
            should_render: false,
            rendered: false,
            scheduled: None,

            drop_hooks: Vec::new(),
            meta: T::Meta::default(),
            //router_state: T::RouterState::default(),
        };
        let mut widget = T::default();
        widget.init(&mut context);
        widget.on_props(&mut context);
        Self { widget, context }
    }

    fn update(&mut self, msg: Self::Message) -> ShouldRender {
        self.context.should_render = false;
        match msg {
            Msg::LiveIncomingWired(envelope) => {
                let req_id = envelope.id;
                if let Some(tag) = self.context.live.registry().tag(&req_id) {
                    match &envelope.data {
                        LiveResponse::WireDone => {
                            self.context.live.registry().remove(&req_id);
                        }
                        LiveResponse::Forwarded(_) => {
                            // TODO: Extract response here and use it in handler
                        }
                    }
                    if let Some(handler) = self.context.live.handler() {
                        if let Err(err) =
                            handler(&mut self.widget, tag.as_ref(), envelope, &mut self.context)
                        {
                            log::error!("Live handler failed: {}", err);
                        }
                    }
                }
            }
            Msg::GraphicsIncoming(response) => {
                if let Some(handler) = self.context.graphics.handler() {
                    if let Err(err) = handler(&mut self.widget, response, &mut self.context) {
                        log::error!("Graphics handler failed: {}", err);
                    }
                }
            }
            Msg::InPlace(mut func) => {
                if let Err(err) = func.handle(&mut self.widget, &mut self.context) {
                    log::error!("Widget callback failed: {}", err);
                }
            }
            Msg::Event(event) => {
                self.widget.on_event(event, &mut self.context);
            }
        }
        self.context.should_render
    }

    fn change(&mut self, props: Self::Properties) -> ShouldRender {
        if props != self.context.props {
            self.context.props = props;
            self.widget.on_props(&mut self.context);
            self.context.should_render
        } else {
            false
        }
    }

    fn view(&self) -> Html {
        self.widget.view(&self.context)
    }

    fn rendered(&mut self, first_render: bool) {
        if first_render {
            self.context.rendered = true;
        }
        if let Err(err) = self.widget.rendered(first_render) {
            log::error!("Rendering failed: {}", err);
        }
    }
}
