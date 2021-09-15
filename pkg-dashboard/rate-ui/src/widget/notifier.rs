use super::{Msg, OnBridgeEvent, Widget, WidgetContext};
use crate::agents::notifier::{Notification, NotifierAgent, NotifierRequest, NotifierResponse};
use yew::Bridge;

pub struct Notifier<'a> {
    link: &'a mut dyn Bridge<NotifierAgent>,
}

impl<'a> Notifier<'a> {
    pub fn listen(&mut self, active: bool) {
        let req = NotifierRequest::Listen(active);
        self.link.send(req);
    }
}

impl<T: Widget> WidgetContext<T> {
    pub fn notifier(&mut self) -> Notifier<'_>
    where
        T: OnBridgeEvent<NotifierAgent>,
    {
        let link = self.notifier.get_mut_linked(&self.link);
        Notifier { link }
    }

    pub fn notify(&mut self, title: String, content: String) {
        let link = self.notifier.activate_link(&self.link);
        let notification = Notification { title, content };
        let req = NotifierRequest::Notify(Some(notification));
        link.send(req);
    }

    pub fn notify_clear(&mut self) {
        let link = self.notifier.activate_link(&self.link);
        let req = NotifierRequest::Notify(None);
        link.send(req);
    }
}

impl<T: Widget> From<NotifierResponse> for Msg<T> {
    fn from(response: NotifierResponse) -> Self {
        Self::NotifierIncoming(response)
    }
}
