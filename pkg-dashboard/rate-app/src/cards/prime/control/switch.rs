use crate::blocks;
use rate_ui::widget::wired_widget::{SingleFlowMeta, SingleFlowProps, WiredWidget};
use rate_ui::widget::{Context, Widget, WidgetRuntime};
use rill_protocol::io::provider::Path;
use rrpack_prime::control::switch::{SwitchAction, SwitchState};
use yew::{html, Html};

pub type SwitchCard = WidgetRuntime<SwitchCardWidget>;

#[derive(Default)]
pub struct SwitchCardWidget {}

impl Widget for SwitchCardWidget {
    type Event = SwitchAction;
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
        ctx.do_action(event);
    }

    fn view(&self, ctx: &Context<Self>) -> Html {
        let body = {
            if let Some(state) = ctx.meta().state() {
                let turn_on = !state.turned_on;
                html! {
                    <div class="form-check form-switch d-flex flex-row justify-content-center"
                        onclick=ctx.event(turn_on)
                        >
                        <input class="form-check-input click" type="checkbox" checked=state.turned_on />
                    </div>
                    //<label class="form-check-label">{ &state.spec.label }</label>
                }
            } else {
                blocks::spinner("Connecting...")
            }
        };
        html! {
            <div yew=module_path!()>
                { body }
            </div>
        }
    }
}

impl WiredWidget<SingleFlowMeta<Self>> for SwitchCardWidget {
    type Flow = SwitchState;

    fn state_changed(&mut self, _reloaded: bool, ctx: &mut Context<Self>) {
        ctx.redraw();
    }
}
