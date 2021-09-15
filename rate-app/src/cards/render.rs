use once_cell::sync::Lazy;
use rate_ui::widget::wired_widget::SingleFlowProps;
use rill_protocol::flow::core::Flow;
use rill_protocol::io::provider::{Path, StreamType};
use rrpack_prime::manifest::layouts::layout::Size;
use std::collections::HashMap;
use yew::{html, Component, Html};

pub type RenderFn = &'static dyn RenderFunc;

pub trait RenderFunc: Send + Sync {
    fn render(&self, path: &Path) -> Html;
}

impl<T> RenderFunc for T
where
    T: Fn(&Path) -> Html,
    T: Sync + Send,
{
    fn render(&self, path: &Path) -> Html {
        (self)(path)
    }
}

fn render_card<T, M>(path: &Path) -> Html
where
    T: Component<Message = M, Properties = SingleFlowProps>,
{
    html! {
        <T path=path.clone() />
    }
}

fn render_default(path: &Path) -> Html {
    html! {
        <div class="d-flex flex-row align-items-center">
            <div class="text-center p-1 mt-1 fw-bold">{ "No render!" }</div>
            <div class="text-center p-1">{ path }</div>
        </div>
    }
}

#[derive(Clone)]
pub struct RenderRule {
    pub render: RenderFn,
    pub size: Size,
}

impl RenderRule {
    fn new<T, M>(width: u32, height: u32) -> Self
    where
        T: Component<Message = M, Properties = SingleFlowProps>,
        M: 'static,
    {
        Self {
            render: &render_card::<T, _>,
            size: Size { width, height },
        }
    }
}

pub const RENDER_DEFAULT: RenderRule = RenderRule {
    render: &render_default,
    size: Size {
        width: 100,
        height: 50,
    },
};

pub static RENDERS: Lazy<HashMap<StreamType, RenderRule>> = Lazy::new(preffered_sizes);

fn preffered_sizes() -> HashMap<StreamType, RenderRule> {
    use super::prime;
    use rrpack_prime::{control, transparent, visual};
    let mut preffered_sizes: HashMap<StreamType, RenderRule> = HashMap::new();

    preffered_sizes.insert(
        transparent::alert::AlertState::stream_type(),
        RenderRule::new::<prime::transparent::AlertCard, _>(100, 100),
    );

    preffered_sizes.insert(
        control::click::ClickState::stream_type(),
        RenderRule::new::<prime::control::ClickCard, _>(150, 100),
    );
    preffered_sizes.insert(
        control::input::InputState::stream_type(),
        RenderRule::new::<prime::control::InputCard, _>(300, 100),
    );
    preffered_sizes.insert(
        control::selector::SelectorState::stream_type(),
        RenderRule::new::<prime::control::SelectorCard, _>(300, 100),
    );
    preffered_sizes.insert(
        control::slider::SliderState::stream_type(),
        RenderRule::new::<prime::control::SliderCard, _>(300, 100),
    );
    preffered_sizes.insert(
        control::switch::SwitchState::stream_type(),
        RenderRule::new::<prime::control::SwitchCard, _>(150, 100),
    );

    preffered_sizes.insert(
        visual::board::BoardState::stream_type(),
        RenderRule::new::<prime::visual::BoardCard, _>(450, 300),
    );
    preffered_sizes.insert(
        visual::counter::CounterState::stream_type(),
        RenderRule::new::<prime::visual::CounterCard, _>(300, 100),
    );
    preffered_sizes.insert(
        visual::gauge::GaugeState::stream_type(),
        RenderRule::new::<prime::visual::GaugeCard, _>(300, 100),
    );
    preffered_sizes.insert(
        visual::histogram::HistogramState::stream_type(),
        RenderRule::new::<prime::visual::HistogramCard, _>(450, 300),
    );
    preffered_sizes.insert(
        visual::live_logs::LiveLogsState::stream_type(),
        RenderRule::new::<prime::visual::LiveLogsCard, _>(450, 200),
    );
    preffered_sizes.insert(
        visual::live_text::LiveTextState::stream_type(),
        RenderRule::new::<prime::visual::LiveTextCard, _>(450, 200),
    );
    preffered_sizes.insert(
        visual::pulse::PulseState::stream_type(),
        RenderRule::new::<prime::visual::PulseCard, _>(450, 300),
    );
    preffered_sizes.insert(
        visual::table::TableState::stream_type(),
        RenderRule::new::<prime::visual::TableCard, _>(800, 400),
    );

    preffered_sizes
}
