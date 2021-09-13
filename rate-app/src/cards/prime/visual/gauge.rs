use crate::blocks;
use rate_ui::widget::wired_widget::{SingleFlowMeta, SingleFlowProps, WiredWidget};
use rate_ui::widget::{Context, Widget, WidgetRuntime};
use rill_protocol::io::provider::Path;
use rill_protocol::range::{Pct, Range};
use rrpack_prime::visual::gauge::GaugeState;
use yew::{html, Html};

pub type GaugeCard = WidgetRuntime<GaugeCardWidget>;

#[derive(Default)]
pub struct GaugeCardWidget {}

impl Widget for GaugeCardWidget {
    type Event = ();
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

    fn view(&self, ctx: &Context<Self>) -> Html {
        let body = {
            if let Some(state) = ctx.meta().state() {
                /*
                    <div class="w-100 p-4 text-end row mt-5">
                        <h2>{ state.stat.value }</h2>
                    </div>
                */
                if let Some(value) = state.value {
                    let min = state.spec.range.min.min(state.abs_min);
                    let max = state.spec.range.max.max(state.abs_max);
                    let range = Range::new(min, max);
                    let pct = Pct::from_range(value, &range);
                    let width = format!("width: {}%;", pct.to_cent());
                    html! {
                        <>
                            <div class="progress">
                                <div class="progress-bar" style=width></div>
                            </div>
                            <div class="d-flex flex-row justify-content-between">
                                <div>{ min }</div>
                                <div>{ value }</div>
                                <div>{ max }</div>
                            </div>
                        </>
                    }
                } else {
                    let width = "width: 100%;";
                    html! {
                        <>
                            <div class="progress">
                                <div class="progress-bar progress-bar-striped progress-bar-animated bg-secondary" style=width></div>
                            </div>
                            <div class="text-secondary text-center">{ "No data yet..." }</div>
                        </>
                    }
                }
            } else {
                blocks::spinner("Connecting...")
            }
        };
        html! {
            <div yew=module_path!() class="flex-grow-1 p-3 d-flex flex-column justify-content-center align-items-stretch">
                { body }
            </div>
        }
    }
}

impl WiredWidget<SingleFlowMeta<Self>> for GaugeCardWidget {
    type Flow = GaugeState;

    fn state_changed(&mut self, _reloaded: bool, ctx: &mut Context<Self>) {
        ctx.redraw();
    }
}
