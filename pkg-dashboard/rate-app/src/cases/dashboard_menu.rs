use rate_ui::agents::live::LiveAgent;
use rate_ui::widget::{Context, OnWireEvent, Widget, WidgetRuntime};
use yew::{html, Html};

pub type DashboardMenu = WidgetRuntime<DashboardMenuWidget>;

#[derive(Default)]
pub struct DashboardMenuWidget {}

impl Widget for DashboardMenuWidget {
    type Event = ();
    type Tag = ();
    type Properties = ();
    type Meta = ();

    fn init(&mut self, _ctx: &mut Context<Self>) {}

    fn view(&self, _ctx: &Context<Self>) -> Html {
        html! {
            <>
                <super::DashboardSelector />
                <super::TabSelector />
            </>
        }
    }
}

impl OnWireEvent<LiveAgent> for DashboardMenuWidget {}
