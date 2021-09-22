use super::state::{SceneState, SCENE};
use anyhow::Error;
use rate_ui::shared_object::{DataChanged, SharedObject};
use rate_ui::widget::{Context, NotificationHandler, Widget, WidgetRuntime};
use rill_protocol::io::provider::EntryId;
use std::collections::BTreeSet;
use yew::{html, Html};

pub type TopSelector = WidgetRuntime<TopSelectorWidget>;

pub struct TopSelectorWidget {
    scene: SharedObject<SceneState>,
}

impl Default for TopSelectorWidget {
    fn default() -> Self {
        Self {
            scene: SCENE.with(SharedObject::clone),
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
        self.scene.subscribe(ctx);
    }

    fn on_event(&mut self, event: Self::Event, _ctx: &mut Context<Self>) {
        match event {
            Msg::SelectDashboard(dashboard) => {
                let mut state = self.scene.write();
                state.selected_layout = dashboard;
            }
        }
    }

    fn view(&self, ctx: &Context<Self>) -> Html {
        let state = self.scene.read();
        let dashboards = state.layouts.keys().cloned().collect::<BTreeSet<_>>();
        html! {
            <nav yew=module_path!() class="nav">
                { for dashboards.into_iter().map(|entry_id| self.render_item(entry_id, ctx)) }
            </nav>
        }
    }
}

impl TopSelectorWidget {
    fn render_item(&self, entry_id: EntryId, ctx: &Context<Self>) -> Html {
        let caption = entry_id.to_string();
        let dashboard = Some(entry_id);
        let state = self.scene.read();
        let selected = dashboard == state.selected_layout.clone();
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

impl NotificationHandler<DataChanged<SceneState>> for TopSelectorWidget {
    fn handle(
        &mut self,
        _event: DataChanged<SceneState>,
        ctx: &mut Context<Self>,
    ) -> Result<(), Error> {
        ctx.redraw();
        Ok(())
    }
}
