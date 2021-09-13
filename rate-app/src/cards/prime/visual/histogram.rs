use crate::blocks;
use ordered_float::OrderedFloat;
use rate_ui::widget::wired_widget::{SingleFlowMeta, SingleFlowProps, WiredWidget};
use rate_ui::widget::{Context, Widget, WidgetRuntime};
use rill_protocol::io::provider::Path;
use rrpack_prime::visual::histogram::{HistogramState, Stat};
use yew::{html, Html};

pub type HistogramCard = WidgetRuntime<HistogramCardWidget>;

#[derive(Default)]
pub struct HistogramCardWidget {}

impl Widget for HistogramCardWidget {
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
                html! {
                    <div class="d-flex flex-column w-100 py-3 px-4 overflow-auto">
                        { for state.buckets.iter().map(|(key, value)| self.render_item(key, value)) }
                    </div>
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

impl HistogramCardWidget {
    fn render_item(&self, key: &OrderedFloat<f64>, stat: &Stat) -> Html {
        html! {
            <div class="d-flex flex-row justify-content-between">
                <div class="fw-bold">{ key }</div>
                <div>{ stat.count }</div>
            </div>
        }
    }
}

impl WiredWidget<SingleFlowMeta<Self>> for HistogramCardWidget {
    type Flow = HistogramState;

    fn state_changed(&mut self, _reloaded: bool, ctx: &mut Context<Self>) {
        ctx.redraw();
    }
}
