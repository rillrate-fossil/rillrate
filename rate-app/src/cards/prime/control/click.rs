use crate::blocks;
use rate_ui::widget::wired_widget::{SingleFlowMeta, SingleFlowProps, WiredWidget};
use rate_ui::widget::{Context, Widget, WidgetRuntime};
use rill_protocol::io::provider::Path;
use rrpack_prime::control::click::{ClickAction, ClickState};
use yew::{html, Html};

pub type ClickCard = WidgetRuntime<ClickCardWidget>;

#[derive(Default)]
pub struct ClickCardWidget {}

impl Widget for ClickCardWidget {
    type Event = ClickAction;
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
                html! {
                    <button class="btn btn-primary click"
                        onclick=ctx.event(())
                        >{ &state.spec.label }</button>
                }
            } else {
                blocks::spinner("Connecting...")
            }
        };
        html! {
            <div yew=module_path!() class="center">
                { body }
            </div>
        }
    }
}

impl WiredWidget<SingleFlowMeta<Self>> for ClickCardWidget {
    type Flow = ClickState;

    fn state_changed(&mut self, _reloaded: bool, ctx: &mut Context<Self>) {
        ctx.redraw();
    }
}
