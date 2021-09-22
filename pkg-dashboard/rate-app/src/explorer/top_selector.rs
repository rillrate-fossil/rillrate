use super::state::{DashboardState, PATHS};
use anyhow::Error;
use rate_ui::shared_object::{DataChanged, SharedObject};
use rate_ui::widget::{Context, NotificationHandler, Widget, WidgetRuntime};
use rill_protocol::io::provider::EntryId;
use yew::{html, Html};

pub type TopSelector = WidgetRuntime<TopSelectorWidget>;

pub struct TopSelectorWidget {
    paths: SharedObject<DashboardState>,
}

impl Default for TopSelectorWidget {
    fn default() -> Self {
        Self {
            paths: PATHS.with(SharedObject::clone),
        }
    }
}

#[derive(Clone)]
pub enum Msg {
    SelectPackage(Option<EntryId>),
}

impl Widget for TopSelectorWidget {
    type Event = Msg;
    type Tag = ();
    type Properties = ();
    type Meta = ();

    fn init(&mut self, ctx: &mut Context<Self>) {
        self.paths.subscribe(ctx);
    }

    fn on_event(&mut self, event: Self::Event, _ctx: &mut Context<Self>) {
        match event {
            Msg::SelectPackage(package) => {
                let mut state = self.paths.write();
                state.selection.selected_package = package;
                state.selection.selected_dashboard.take();
            }
        }
    }

    fn view(&self, ctx: &Context<Self>) -> Html {
        let state = self.paths.read();
        let packages = state.structure.packages.keys().cloned();
        html! {
            <nav yew=module_path!() class="nav">
                // TODO: Add unassigned option if the package exists
                // TODO: Filter unassigned
                { for packages.map(|entry_id| self.render_item(entry_id, ctx)) }
            </nav>
        }
    }
}

impl TopSelectorWidget {
    fn render_item(&self, entry_id: EntryId, ctx: &Context<Self>) -> Html {
        let state = self.paths.read();
        let caption = entry_id.to_string();
        let package = Some(entry_id);
        let selected = package == state.selection.selected_package;
        let (class, event) = {
            if selected {
                ("nav-link link-primary active", None)
            } else {
                ("nav-link link-secondary", package)
            }
        };
        html! {
            <div class="nav-item click">
                <a class=class
                    onclick=ctx.event(Msg::SelectPackage(event))
                    >{ caption }</a>
            </div>
        }
    }
}

impl NotificationHandler<DataChanged<DashboardState>> for TopSelectorWidget {
    fn handle(
        &mut self,
        _event: DataChanged<DashboardState>,
        ctx: &mut Context<Self>,
    ) -> Result<(), Error> {
        ctx.redraw();
        Ok(())
    }
}
