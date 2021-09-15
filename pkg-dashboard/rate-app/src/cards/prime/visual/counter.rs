use crate::blocks;
use rate_ui::widget::wired_widget::{SingleFlowMeta, SingleFlowProps, WiredWidget};
use rate_ui::widget::{Context, Widget, WidgetRuntime};
use rill_protocol::io::provider::Path;
use rrpack_prime::visual::counter::CounterState;
use yew::{html, Html};

pub type CounterCard = WidgetRuntime<CounterCardWidget>;

#[derive(Default)]
pub struct CounterCardWidget {}

impl Widget for CounterCardWidget {
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
                let (m, t, o) = splitnum(state.total);
                html! {
                    <div class="w-100 p-2 text-end row">
                        <div class="col-4">
                            <h2 class="like-mono">{ m }</h2>
                            <div class="text-secondary">{ "mil" }</div>
                        </div>
                        <div class="col-4">
                            <h2 class="like-mono">{ t }</h2>
                            <div class="text-secondary">{ "thous" }</div>
                        </div>
                        <div class="col-4">
                            <h2 class="like-mono">{ o }</h2>
                            <div class="text-secondary">{ "ones" }</div>
                        </div>
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

fn splitnum(value: i64) -> (String, String, String) {
    let s = value.to_string();
    let mut chars = s.chars().rev();
    let mut ones: String = (&mut chars).take(3).collect();
    let mut thousands: String = (&mut chars).take(3).collect();
    let mut millions: String = chars.collect();
    const NO_PRINT: char = '_';
    if ones.is_empty() {
        ones.push(NO_PRINT);
    }
    if thousands.is_empty() {
        thousands.push(NO_PRINT);
    }
    if millions.is_empty() {
        millions.push(NO_PRINT);
    }
    (rev(millions), rev(thousands), rev(ones))
}

fn rev(s: String) -> String {
    s.chars().rev().collect()
}

impl WiredWidget<SingleFlowMeta<Self>> for CounterCardWidget {
    type Flow = CounterState;

    fn state_changed(&mut self, _reloaded: bool, ctx: &mut Context<Self>) {
        ctx.redraw();
    }
}
