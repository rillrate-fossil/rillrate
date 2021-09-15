use crate::blocks;
use rate_ui::widget::wired_widget::{SingleFlowMeta, SingleFlowProps, WiredWidget};
use rate_ui::widget::{Context, Widget, WidgetRuntime};
use rill_protocol::io::provider::Path;
use rrpack_prime::control::input::InputState;
use yew::{html, Html, InputData};

pub type InputCard = WidgetRuntime<InputCardWidget>;

#[derive(Default)]
pub struct InputCardWidget {}

pub enum Msg {
    Update(String),
}

impl Widget for InputCardWidget {
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
            Msg::Update(value) => {
                ctx.do_action(value);
            }
        }
    }

    fn view(&self, ctx: &Context<Self>) -> Html {
        let body = {
            if let Some(_state) = ctx.meta().state() {
                // TODO: Use placeholder from the state
                // TODO: Override type with Spec! (password, etc)
                html! {
                    <input type="text" class="form-control" oninput=ctx.callback(|data: InputData| Msg::Update(data.value)) />
                }
            } else {
                blocks::spinner("Connecting...")
            }
        };
        html! {
            <div yew=module_path!() class="center align-items-stretch">
                { body }
            </div>
        }
    }
}

impl WiredWidget<SingleFlowMeta<Self>> for InputCardWidget {
    type Flow = InputState;

    fn state_changed(&mut self, _reloaded: bool, ctx: &mut Context<Self>) {
        ctx.redraw();
    }
}
