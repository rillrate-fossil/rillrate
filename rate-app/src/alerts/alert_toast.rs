use super::state::{TimedAlert, ToastState, ALERTS};
use anyhow::Error;
use rate_ui::shared_object::{DataChanged, SharedObject};
use rate_ui::widget::{Context, NotificationHandler, Widget, WidgetRuntime};
use std::time::Duration;
use timeago::Formatter;
use yew::{html, Html};

const REFRESH_INTERVAL: u64 = 1_000;
const RETAIN_INTERVAL: u64 = 5_600;

pub type AlertToast = WidgetRuntime<AlertToastWidget>;

pub struct AlertToastWidget {
    /// `BTreeSet` is used to throttle duplicated messages.
    formatter: Formatter,
    alerts: SharedObject<ToastState>,
}

impl Default for AlertToastWidget {
    fn default() -> Self {
        Self {
            formatter: Default::default(),
            alerts: ALERTS.with(SharedObject::clone),
        }
    }
}

#[derive(Clone)]
pub enum Msg {
    Hide(usize),
    // TODO: Use `NotificationHandler` instead
    TimerRefresh,
}

impl Widget for AlertToastWidget {
    type Event = Msg;
    type Tag = ();
    type Properties = ();
    type Meta = ();

    fn init(&mut self, ctx: &mut Context<Self>) {
        self.alerts.subscribe(ctx);
        self.reschedule(ctx);
    }

    fn on_event(&mut self, event: Self::Event, ctx: &mut Context<Self>) {
        let mut state = self.alerts.write();
        match event {
            Msg::Hide(idx) => {
                state.alerts.remove(idx);
                ctx.redraw();
            }
            Msg::TimerRefresh => {
                let deadline = js_sys::Date::now() as u64 - RETAIN_INTERVAL;
                state.alerts.retain(|alert| alert.ms > deadline);
                drop(state);
                self.reschedule(ctx);
                ctx.redraw();
            }
        }
    }

    fn view(&self, ctx: &Context<Self>) -> Html {
        let state = self.alerts.read();
        let timestamp = Duration::from_millis(js_sys::Date::now() as u64);
        html! {
          <div class="position-fixed bottom-0 end-0 p-3" style="z-index: 50000">
            <div class="toast-container">
              { for state.alerts.iter().enumerate().take(10).map(|(idx, msg)| self.render_alert(idx, msg, timestamp, ctx)) }
            </div>
          </div>
        }
    }
}

impl AlertToastWidget {
    fn render_alert(
        &self,
        idx: usize,
        item: &TimedAlert,
        from: Duration,
        ctx: &Context<Self>,
    ) -> Html {
        let ago = self
            .formatter
            .convert(from - Duration::from_millis(item.ms));
        html! {
          <div class="toast show">
            <div class="toast-header">
              //<img src="..." class="rounded me-2" alt="..." />
              <strong class="me-auto">{ &item.origin }</strong>
              <small>{ ago }</small>
              <button type="button" class="btn-close"
                onclick=ctx.event(Msg::Hide(idx))
              ></button>
            </div>
            <div class="toast-body">{ &item.message }</div>
          </div>
        }
    }

    // TODO: Move that to `Context`
    fn reschedule(&mut self, ctx: &mut Context<Self>) {
        let state = self.alerts.read();
        if !state.alerts.is_empty() {
            // It drop's the latest `schedule` (refreshes)
            ctx.schedule(REFRESH_INTERVAL, Msg::TimerRefresh);
        } else {
            // Not necessary, but it's better to free memory
            ctx.unschedule();
        }
    }
}

impl NotificationHandler<DataChanged<ToastState>> for AlertToastWidget {
    fn handle(
        &mut self,
        _event: DataChanged<ToastState>,
        ctx: &mut Context<Self>,
    ) -> Result<(), Error> {
        self.reschedule(ctx);
        ctx.redraw();
        Ok(())
    }
}
