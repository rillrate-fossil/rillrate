use crate::alerts::state::{TimedAlert, ALERTS};
use rate_ui::shared_object::SharedObject;
use rate_ui::widget::wired_widget::{SingleFlowMeta, SingleFlowProps, WiredWidget};
use rate_ui::widget::{Context, Widget, WidgetRuntime};
use rill_protocol::io::provider::Path;
use rrpack_prime::transparent::alert::{AlertEvent, AlertState};
use yew::{classes, html, Classes, Html};

pub type AlertCard = WidgetRuntime<AlertCardWidget>;

#[derive(Default)]
pub struct AlertCardWidget {
    // TODO: Add ignored count
    silent: bool,
}

#[derive(Debug, Clone)]
pub enum Msg {
    Toggle,
}

impl Widget for AlertCardWidget {
    type Event = Msg;
    type Tag = Option<Path>;
    type Properties = SingleFlowProps;
    type Meta = SingleFlowMeta<Self>;

    fn init(&mut self, ctx: &mut Context<Self>) {
        self.on_props(ctx);
    }

    fn on_props(&mut self, ctx: &mut Context<Self>) {
        let path = ctx.properties().path.clone().of_server();
        ctx.rewire(path);
    }

    fn on_event(&mut self, event: Self::Event, ctx: &mut Context<Self>) {
        match event {
            Msg::Toggle => {
                self.silent = !self.silent;
                ctx.redraw();
            }
        }
    }

    fn view(&self, ctx: &Context<Self>) -> Html {
        let color = {
            if self.silent {
                "btn-dark"
            } else {
                "btn-warning"
            }
        };
        let (silent, bell) = if self.silent {
            (None, "bi-bell")
        } else {
            (Some("active"), "bi-bell-fill")
        };
        let body = {
            html! {
                <button type="button" class=classes!(Classes::from("btn btn-sm"), color, silent)
                    onclick=ctx.event(Msg::Toggle)
                >
                    { "Alerts: " }<i class=bell></i>
                </button>
            }
        };
        html! {
            <div yew=module_path!() class="text-center pt-3">
                { body }
            </div>
        }
    }
}

impl WiredWidget<SingleFlowMeta<Self>> for AlertCardWidget {
    type Flow = AlertState;

    fn state_changed(&mut self, _reloaded: bool, ctx: &mut Context<Self>) {
        ctx.redraw();
    }

    fn state_update(
        &mut self,
        tag: &Path,
        event: &AlertEvent,
        _reloaded: &mut bool,
        _ctx: &mut Context<Self>,
    ) {
        match event {
            AlertEvent::Notify { text } => {
                if !self.silent {
                    let alerts = ALERTS.with(SharedObject::clone);
                    let mut state = alerts.write();
                    let origin = tag.iter().last().cloned().unwrap_or_default();
                    let alert = TimedAlert::new(origin.into(), text.clone());
                    state.alerts.push_back(alert);
                }
            }
        }
    }
}
