use crate::blocks;
use rate_ui::widget::wired_widget::{SingleFlowMeta, SingleFlowProps, WiredWidget};
use rate_ui::widget::{Context, Widget, WidgetRuntime};
use rill_protocol::io::provider::Path;
use rrpack_prime::control::selector::SelectorState;
use yew::{html, Html};
use yew_components::Select;

pub type SelectorCard = WidgetRuntime<SelectorCardWidget>;

#[derive(Default)]
pub struct SelectorCardWidget {}

pub enum Msg {
    Select(String),
}

impl Widget for SelectorCardWidget {
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
            Msg::Select(value) => {
                ctx.do_action(Some(value));
            }
        }
    }

    fn view(&self, ctx: &Context<Self>) -> Html {
        let body = {
            if let Some(state) = ctx.meta().state() {
                html! {
                    <Select<String>
                        class="form-select click"
                        options=state.spec.options.clone()
                        selected=state.selected.clone()
                        on_change=ctx.callback(Msg::Select)
                    />
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

impl WiredWidget<SingleFlowMeta<Self>> for SelectorCardWidget {
    type Flow = SelectorState;

    fn state_changed(&mut self, _reloaded: bool, ctx: &mut Context<Self>) {
        ctx.redraw();
    }
}
