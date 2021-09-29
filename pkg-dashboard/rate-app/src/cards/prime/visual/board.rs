use crate::blocks;
use rate_ui::widget::wired_widget::{SingleFlowMeta, SingleFlowProps, WiredWidget};
use rate_ui::widget::{Context, Widget, WidgetRuntime};
use rill_protocol::io::provider::Path;
use rrpack_prime::visual::board::BoardState;
use yew::{html, Html};

pub type BoardCard = WidgetRuntime<BoardCardWidget>;

#[derive(Default)]
pub struct BoardCardWidget {}

impl Widget for BoardCardWidget {
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
                        { for state.map.iter().map(|(key, value)| self.render_item(key, value)) }
                    </div>
                }
            } else {
                blocks::spinner("Connecting...")
            }
        };
        html! {
            <div yew=module_path!() class="overflow-auto pe-3" style="height: 100%; width: 100%;">
                { body }
            </div>
        }
    }
}

impl BoardCardWidget {
    fn render_item(&self, key: &str, value: &str) -> Html {
        html! {
            <div class="d-flex flex-row justify-content-between">
                <div class="fw-bold">{ key }</div>
                <div class="text-end">{ value }</div>
            </div>
        }
    }
}

impl WiredWidget<SingleFlowMeta<Self>> for BoardCardWidget {
    type Flow = BoardState;

    fn state_changed(&mut self, _reloaded: bool, ctx: &mut Context<Self>) {
        ctx.redraw();
    }
}
