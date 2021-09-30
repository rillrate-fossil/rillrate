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
            if let Some(state) = ctx.meta().state() {
                let placeholder = state.spec.placeholder.clone();
                if state.spec.wide {
                    let style = if state.spec.password {
                        "color: transparent;text-shadow: 0 0 8px rgba(0,0,0,0.5);"
                    } else {
                        ""
                    };
                    html! {
                        <div class="form-floating">
                            <textarea
                                class="form-control"
                                placeholder=placeholder
                                style=style
                                oninput=ctx.callback(|data: InputData| Msg::Update(data.value))
                            />
                            <label for="floatingTextarea">{ &state.spec.label }</label>
                        </div>
                    }
                } else {
                    let typ = if state.spec.password {
                        "password"
                    } else {
                        "text"
                    };
                    html! {
                        <div class="form-floating">
                            <input
                                type=typ
                                class="form-control"
                                placeholder=placeholder
                                oninput=ctx.callback(|data: InputData| Msg::Update(data.value))
                            />
                            <label for="floatingInput">{ &state.spec.label }</label>
                        </div>
                    }
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

impl WiredWidget<SingleFlowMeta<Self>> for InputCardWidget {
    type Flow = InputState;

    fn state_changed(&mut self, _reloaded: bool, ctx: &mut Context<Self>) {
        ctx.redraw();
    }
}
