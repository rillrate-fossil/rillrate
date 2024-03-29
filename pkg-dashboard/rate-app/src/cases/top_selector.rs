use super::state::{CasesState, CASES};
use anyhow::Error;
use rate_ui::shared_object::{DataChanged, SharedObject};
use rate_ui::widget::{Context, NotificationHandler, Widget, WidgetRuntime};
use rill_protocol::io::provider::EntryId;
use yew::{html, Html};

pub type TopSelector = WidgetRuntime<TopSelectorWidget>;

pub struct TopSelectorWidget {
    cases: SharedObject<CasesState>,
}

impl Default for TopSelectorWidget {
    fn default() -> Self {
        Self {
            cases: CASES.with(SharedObject::clone),
        }
    }
}

#[derive(Clone)]
pub enum Msg {
    SelectDashboard(Option<EntryId>),
}

impl Widget for TopSelectorWidget {
    type Event = Msg;
    type Tag = ();
    type Properties = ();
    type Meta = ();

    fn init(&mut self, ctx: &mut Context<Self>) {
        self.cases.subscribe(ctx);
    }

    fn on_event(&mut self, event: Self::Event, _ctx: &mut Context<Self>) {
        match event {
            Msg::SelectDashboard(dashboard) => {
                let mut state = self.cases.write();
                state.selected_layout = dashboard;
            }
        }
    }

    fn view(&self, ctx: &Context<Self>) -> Html {
        let state = self.cases.read();
        let dashboards = state.structure.get_packages();
        html! {
            <nav yew=module_path!() class="nav">
                { for dashboards.into_iter().map(|entry_id| self.render_item(entry_id, ctx)) }
            </nav>
        }
    }
}

impl TopSelectorWidget {
    fn render_item(&self, entry_id: &EntryId, ctx: &Context<Self>) -> Html {
        let caption = entry_id.to_string();
        let dashboard = Some(entry_id);
        let state = self.cases.read();
        let selected = dashboard == state.selected_layout.as_ref();
        let (class, event) = {
            if selected {
                ("nav-link link-primary active", None)
            } else {
                ("nav-link link-secondary", dashboard)
            }
        };
        html! {
            <div class="nav-item click">
                <a class=class
                    onclick=ctx.event(Msg::SelectDashboard(event.cloned()))
                    >{ caption }</a>
            </div>
        }
    }
}

impl NotificationHandler<DataChanged<CasesState>> for TopSelectorWidget {
    fn handle(
        &mut self,
        _event: DataChanged<CasesState>,
        ctx: &mut Context<Self>,
    ) -> Result<(), Error> {
        ctx.redraw();
        Ok(())
    }
}
