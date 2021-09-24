use crate::blocks;
use rate_ui::widget::wired_widget::{SingleFlowMeta, SingleFlowProps, WiredWidget};
use rate_ui::widget::{Context, Widget, WidgetRuntime};
use rill_protocol::io::provider::Path;
use rrpack_prime::control::slider::SliderState;
use web_sys::HtmlElement;
use yew::{html, ChangeData, Html, InputData, NodeRef};

pub type SliderCard = WidgetRuntime<SliderCardWidget>;

#[derive(Default)]
pub struct SliderCardWidget {
    /// Don't redraw on changing
    changing: bool,
    temp_value: Option<f64>,
    current_value: NodeRef,
}

#[derive(Debug)]
pub enum Msg {
    BeginChange,
    Change(ChangeData),
    Input(InputData),
    EndChange,
}

impl SliderCardWidget {
    fn send(&mut self, ctx: &mut Context<Self>) {
        if let Some(new_value) = self.temp_value {
            ctx.do_action(new_value);
        }
    }

    fn parse(&mut self, value: &str) {
        match value.parse() {
            Ok(new_value) => {
                self.temp_value = Some(new_value);
            }
            Err(err) => {
                log::error!("Can't parse slider value: {}", err);
            }
        }
    }

    fn update_current(&mut self, value: &str) {
        if let Some(element) = self.current_value.cast::<HtmlElement>() {
            element.set_inner_text(value);
        }
    }
}

impl Widget for SliderCardWidget {
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
            Msg::Change(ChangeData::Value(data)) => {
                self.parse(&data);
                self.send(ctx);
                self.changing = false;
            }
            Msg::Input(input) => {
                self.update_current(&input.value);
                self.parse(&input.value);
                let instant_send = ctx
                    .meta()
                    .state()
                    .map(|state| state.spec.instant)
                    .unwrap_or_default();
                if instant_send {
                    self.send(ctx);
                }
            }
            Msg::Change(other) => {
                log::error!("Unsupported event for slider: {:?}", other);
            }
            Msg::BeginChange => {
                self.changing = true;
            }
            Msg::EndChange => {
                self.changing = false;
            }
        }
    }

    fn view(&self, ctx: &Context<Self>) -> Html {
        let body = {
            if let Some(state) = ctx.meta().state() {
                html! {
                    <div class="center">
                        //<label class="form-label">{ &state.spec.label }</label>
                        <input type="range" class="form-range click"
                            min=state.spec.min.to_string()
                            max=state.spec.max.to_string()
                            step=state.spec.step.to_string()
                            value={self.temp_value.unwrap_or(state.value).to_string()}
                            // I used `callback` instead of `event`, because `ChangeData` in not
                            // cloneable and `Msg` poisoned of it.
                            onmousedown=ctx.callback(|_| Msg::BeginChange)
                            ontouchstart=ctx.callback(|_| Msg::BeginChange)
                            onmouseup=ctx.callback(|_| Msg::EndChange)
                            ontouchend=ctx.callback(|_| Msg::EndChange)
                            oninput=ctx.callback(Msg::Input)
                            onchange=ctx.callback(Msg::Change)
                        />
                        <div class="d-flex flex-row justify-content-between">
                            <div>{ state.spec.min }</div>
                            <div ref=self.current_value.clone()>{ state.value }</div>
                            <div>{ state.spec.max }</div>
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

impl WiredWidget<SingleFlowMeta<Self>> for SliderCardWidget {
    type Flow = SliderState;

    fn state_changed(&mut self, _reloaded: bool, ctx: &mut Context<Self>) {
        if !self.changing {
            self.temp_value.take();
            if let Some(state) = ctx.meta().state() {
                self.update_current(&state.value.to_string());
            }
            ctx.redraw();
        }
    }
}
