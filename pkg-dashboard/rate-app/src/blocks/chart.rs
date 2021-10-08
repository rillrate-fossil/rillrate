use crate::canvas::DrawCanvas;
use anyhow::Error;
use rate_ui::agents::graphics::{GraphicsAgent, GraphicsResponse};
use rate_ui::widget::wired_widget::{SingleFlowMeta, SingleFlowProps, WiredWidget};
use rate_ui::widget::{Context, OnBridgeEvent, Widget, WidgetRuntime};
use rill_protocol::flow::core::Flow;
use rill_protocol::io::provider::Path;
use yew::{html, Html};

pub type BasicChart<T> = WidgetRuntime<BasicChartWidget<T>>;

#[derive(Default)]
pub struct BasicChartWidget<T> {
    canvas: DrawCanvas,
    spec: T,
}

impl<T: ChartSpec> Widget for BasicChartWidget<T> {
    type Event = ();
    type Tag = Option<Path>;
    type Properties = SingleFlowProps;
    type Meta = SingleFlowMeta<Self>;

    fn init(&mut self, ctx: &mut Context<Self>) {
        ctx.graphics().on_frame(true);
        ctx.graphics().track_size(self.canvas.node_ref().clone());
        self.on_props(ctx);
    }

    fn on_props(&mut self, ctx: &mut Context<Self>) {
        ctx.rewire(ctx.properties().path.clone());
    }

    fn rendered(&mut self, first: bool) -> Result<(), Error> {
        if first {
            self.canvas.bind()?;
            self.canvas.resize()?;
        }
        Ok(())
    }

    fn view(&self, _ctx: &Context<Self>) -> Html {
        let node_ref = self.canvas.node_ref().clone();
        html! {
            <canvas
                ref=node_ref
                //class="chart"
                // IMPORTANT: `block` helps to avoid continuos height grow on resize
                style="width: 100%; height: 100%; padding: 0; margin: 0; box-sizing: border-box;"
            />
        }
    }
}

impl<T: ChartSpec> WiredWidget<SingleFlowMeta<Self>> for BasicChartWidget<T> {
    type Flow = T::Flow;

    fn state_changed(&mut self, _reloaded: bool, ctx: &mut Context<Self>) {
        if let Some(state) = ctx.meta().state() {
            self.spec.upgrade(state);
        }
    }
}

impl<T: ChartSpec> OnBridgeEvent<GraphicsAgent> for BasicChartWidget<T> {
    fn on_event(&mut self, event: GraphicsResponse, ctx: &mut Context<Self>) -> Result<(), Error> {
        match event {
            GraphicsResponse::SizeChanged(_) => {
                self.canvas.resize()?;
            }
            GraphicsResponse::Frame => {
                if let Some(state) = ctx.meta().state() {
                    self.canvas.resize()?;
                    self.canvas.clear()?;
                    // TODO: Do `upgrade` once only
                    // when I'll use moving x coords for cartesian 2d
                    self.spec.upgrade(state);

                    self.canvas.draw_charts(
                        self.spec.secs(),
                        0,
                        self.spec.y_min(),
                        self.spec.y_max(),
                        self.spec.x_formatter(),
                        self.spec.y_formatter(),
                        self.spec.data(),
                    )?;
                }
            }
        }
        Ok(())
    }
}

pub trait ChartSpec: Default + 'static {
    type Flow: Flow;

    fn upgrade(&mut self, state: &Self::Flow);

    fn secs(&self) -> i64 {
        30
    }

    fn data(&self) -> &Vec<Vec<(i64, f32)>>;

    fn y_min(&self) -> f32 {
        0.0
    }

    fn y_max(&self) -> f32;

    fn x_formatter(&self) -> &Formatter<i64>;

    fn y_formatter(&self) -> &Formatter<f32>;
}

// TODO: Try to use the same `T` type here
pub type Formatter<T> = dyn Fn(&T) -> String;
