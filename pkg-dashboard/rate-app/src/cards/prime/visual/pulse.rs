use crate::blocks;
use crate::blocks::chart::{ChartSpec, Formatter};
use crate::canvas;
use rate_ui::widget::wired_widget::SingleFlowProps;
use rate_ui::widget::{Context, Widget, WidgetRuntime};
use rrpack_prime::visual::pulse::PulseState;
use yew::{html, Html};

pub type PulseCard = WidgetRuntime<PulseCardWidget>;

#[derive(Default)]
pub struct PulseCardWidget {}

impl Widget for PulseCardWidget {
    type Event = ();
    type Tag = ();
    type Properties = SingleFlowProps;
    type Meta = ();

    fn init(&mut self, ctx: &mut Context<Self>) {
        self.on_props(ctx);
    }

    fn on_props(&mut self, ctx: &mut Context<Self>) {
        //let path = ctx.properties().path.clone().of_server();
        //ctx.rewire(path);
        ctx.redraw();
    }

    fn view(&self, ctx: &Context<Self>) -> Html {
        let path = ctx.properties().path.clone().of_server();
        html! {
            <div yew=module_path!() style="width: 100%; height: 100%;">
                <blocks::BasicChart<PulseSpec> path=path />
            </div>
        }
    }
}

/*
impl WiredWidget<SingleFlowMeta<Self>> for PulseCardWidget {
    type Flow = PulseState;

    fn state_changed(&mut self, _reloaded: bool, ctx: &mut Context<Self>) {
        ctx.redraw();
    }
}
*/

#[derive(Default)]
struct PulseSpec {
    retain: i64,
    min: f32,
    max: f32,
    formatter: Option<Box<Formatter<f32>>>,
    lines: Vec<Vec<(i64, f32)>>,
}

// TODO: Replace `ChartSpec` to `struct`
impl ChartSpec for PulseSpec {
    type Flow = PulseState;

    fn upgrade(&mut self, state: &Self::Flow) {
        if self.formatter.is_none() {
            let label = state.spec.label.clone();
            let func =
                move |value: &f32| format!("{} {}", value / label.divisor as f32, label.caption);
            self.formatter = Some(Box::new(func));
        }
        self.retain = state.spec.retain as i64;
        if state.frame.len() > 0 {
            let mut min = f64::MAX;
            let mut max = f64::MIN;
            for item in state.frame.iter() {
                let value = item.event;
                if value < min {
                    min = value;
                }
                if value > max {
                    max = value;
                }
            }
            self.min = state.spec.range.min.min(min) as f32;
            self.max = state.spec.range.max.max(max) as f32;
            // TODO: Avoid using time here!!!
            let x_to = js_sys::Date::now() as i64;
            self.lines.clear();
            let usage = canvas::sustain(
                state.frame.iter().map(|timed_event| {
                    (
                        timed_event.timestamp.0 as i64 - x_to,
                        timed_event.event as f32,
                    )
                }),
                0,
            );
            self.lines.push(usage);
        } else {
            self.min = 0.0;
            self.max = 0.0;
        }
    }

    fn data(&self) -> &Vec<Vec<(i64, f32)>> {
        &self.lines
    }

    fn secs(&self) -> i64 {
        self.retain
    }

    fn y_min(&self) -> f32 {
        self.min
    }

    fn y_max(&self) -> f32 {
        self.max
    }

    fn x_formatter(&self) -> &Formatter<i64> {
        &canvas::formatter_sec
    }

    fn y_formatter(&self) -> &Formatter<f32> {
        if let Some(f) = self.formatter.as_ref() {
            f
        } else {
            &canvas::formatter_plain
        }
    }
}
