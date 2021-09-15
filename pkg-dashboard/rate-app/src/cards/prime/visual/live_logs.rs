use crate::blocks;
use rate_ui::widget::wired_widget::{SingleFlowMeta, SingleFlowProps, WiredWidget};
use rate_ui::widget::{Context, Widget, WidgetRuntime};
use rill_protocol::io::provider::Path;
use rrpack_prime::visual::live_logs::{LiveLogsState, LogRecord};
use yew::{html, Html};

pub type LiveLogsCard = WidgetRuntime<LiveLogsCardWidget>;

#[derive(Default)]
pub struct LiveLogsCardWidget {}

impl Widget for LiveLogsCardWidget {
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
                    //<div class="d-flex flex-column w-100 py-3 px-4 overflow-auto">
                    <table class="table">
                        <thead>
                            <tr>
                                <td width="10%">{ "Module" }</td>
                                <td width="10%">{ "Level" }</td>
                                <td width="10%">{ "Timestamp" }</td>
                                <td width="70%">{ "Text" }</td>
                            </tr>
                        </thead>
                        <tbody>
                            { for state.frame.iter().rev().map(|record| self.render_record(record)) }
                        </tbody>
                    </table>
                    //</div>
                }
            } else {
                blocks::spinner("Connecting...")
            }
        };
        html! {
            <div yew=module_path!() class="overflow-auto p-3" style="height: 100%; width: 100%;">
                { body }
            </div>
        }
    }
}

impl LiveLogsCardWidget {
    fn render_record(&self, record: &LogRecord) -> Html {
        html! {
            <tr>
                <td>{ &record.module }</td>
                <td>{ &record.level }</td>
                <td>{ &record.timestamp }</td>
                <td>{ &record.content }</td>
            </tr>
        }
    }
}

impl WiredWidget<SingleFlowMeta<Self>> for LiveLogsCardWidget {
    type Flow = LiveLogsState;

    fn state_changed(&mut self, _reloaded: bool, ctx: &mut Context<Self>) {
        ctx.redraw();
    }
}
