use super::{Context, Msg, Widget, WidgetRuntime};
use anyhow::{anyhow, Error};
use std::ops::DerefMut;
use yew::worker::Agent;
use yew::{Bridge, Bridged, ComponentLink};

pub trait OnBridgeEvent<T: Agent>: Widget {
    fn on_event(&mut self, _response: T::Output, _ctx: &mut Context<Self>) -> Result<(), Error> {
        let self_type_name = std::any::type_name::<Self>();
        let type_name = std::any::type_name::<T>();
        Err(anyhow!(
            "No implementation for incoming event from the agent: {} of {}.",
            type_name,
            self_type_name
        ))
    }
}

pub type Handler<E, W> = &'static dyn Fn(&mut W, E, &mut Context<W>) -> Result<(), Error>;

pub struct LazyBridge<T: Agent, W: Widget> {
    link: Option<Box<dyn Bridge<T>>>,
    // This filled only if subscribe method called
    handler: Option<Handler<T::Output, W>>,
}

impl<T: Agent, W: Widget> Default for LazyBridge<T, W> {
    fn default() -> Self {
        Self {
            link: None,
            handler: None,
        }
    }
}

impl<T: Agent, W: Widget> LazyBridge<T, W> {
    pub fn activate_link(
        &mut self,
        widget_link: &ComponentLink<WidgetRuntime<W>>,
    ) -> &mut dyn Bridge<T>
    where
        Msg<W>: From<T::Output>,
    {
        if self.link.is_none() {
            let callback = widget_link.callback(Msg::from);
            let link = T::bridge(callback);
            self.link = Some(link);
        }
        self.link.as_mut().map(Box::deref_mut).unwrap()
    }

    pub fn activate_handler(&mut self)
    where
        W: OnBridgeEvent<T>,
    {
        if self.handler.is_none() {
            let handler = &<W as OnBridgeEvent<T>>::on_event;
            self.handler = Some(handler);
        }
    }

    pub fn get_mut_linked(
        &mut self,
        widget_link: &ComponentLink<WidgetRuntime<W>>,
    ) -> &mut dyn Bridge<T>
    where
        Msg<W>: From<T::Output>,
        W: OnBridgeEvent<T>,
    {
        self.activate_handler();
        self.activate_link(widget_link)
    }

    pub fn handler(&self) -> Option<Handler<T::Output, W>> {
        self.handler
    }
}
