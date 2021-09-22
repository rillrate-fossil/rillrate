use super::state::{DashboardState, PATHS};
use anyhow::Error;
use rate_ui::shared_object::{DataChanged, SharedObject};
use rate_ui::widget::{Context, NotificationHandler, Widget, WidgetRuntime};
use rill_protocol::io::provider::EntryId;
use std::collections::BTreeSet;
use yew::{html, Html};

pub type TabSelector = WidgetRuntime<TabSelectorWidget>;

pub struct TabSelectorWidget {
    paths: SharedObject<DashboardState>,
}

impl Default for TabSelectorWidget {
    fn default() -> Self {
        Self {
            paths: PATHS.with(SharedObject::clone),
        }
    }
}

#[derive(Clone)]
pub enum Msg {
    SelectDashboard(Option<EntryId>),
}

impl Widget for TabSelectorWidget {
    type Event = Msg;
    type Tag = ();
    type Properties = ();
    type Meta = ();

    fn init(&mut self, ctx: &mut Context<Self>) {
        self.paths.subscribe(ctx);
    }

    fn on_event(&mut self, event: Self::Event, _ctx: &mut Context<Self>) {
        match event {
            Msg::SelectDashboard(dashboard) => {
                let mut state = self.paths.write();
                state.selection.selected_dashboard = dashboard;
            }
        }
    }

    fn view(&self, ctx: &Context<Self>) -> Html {
        let state = self.paths.read();
        let dashboards = state
            .selection
            .selected_package
            .as_ref()
            .and_then(|package| state.structure.packages.get(package))
            .map(|dashboards| dashboards.keys().cloned().collect::<BTreeSet<_>>())
            .unwrap_or_default();
        html! {
            <nav yew=module_path!() class="nav">
                { for dashboards.into_iter().map(|entry_id| self.render_item(entry_id, ctx)) }
            </nav>
        }
    }
}

impl TabSelectorWidget {
    fn render_item(&self, entry_id: EntryId, ctx: &Context<Self>) -> Html {
        let state = self.paths.read();
        let caption = entry_id.to_string();
        let dashboard = Some(entry_id);
        let selected = dashboard == state.selection.selected_dashboard;
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
                    onclick=ctx.event(Msg::SelectDashboard(event))
                    >{ caption }</a>
            </div>
        }
    }
}

impl NotificationHandler<DataChanged<DashboardState>> for TabSelectorWidget {
    fn handle(
        &mut self,
        _event: DataChanged<DashboardState>,
        ctx: &mut Context<Self>,
    ) -> Result<(), Error> {
        ctx.redraw();
        Ok(())
    }
}
