use super::state::{SceneState, SCENE};
use anyhow::Error;
use rate_ui::shared_object::{DataChanged, SharedObject};
use rate_ui::widget::{Context, NotificationHandler, Widget, WidgetRuntime};
use rill_protocol::io::provider::EntryId;
use yew::{html, Html};

pub type TabSelector = WidgetRuntime<TabSelectorWidget>;

pub struct TabSelectorWidget {
    paths: SharedObject<SceneState>,
}

impl Default for TabSelectorWidget {
    fn default() -> Self {
        Self {
            paths: SCENE.with(SharedObject::clone),
        }
    }
}

#[derive(Clone)]
pub enum Msg {
    SelectTab(Option<EntryId>),
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
            Msg::SelectTab(layout) => {
                let mut state = self.paths.write();
                state.selected_layout = layout;
                state.selected_tab.take();
            }
        }
    }

    fn view(&self, ctx: &Context<Self>) -> Html {
        let state = self.paths.read();
        let tabls = state.layouts.keys().cloned();
        html! {
            <nav yew=module_path!() class="nav">
                // TODO: Add unassigned option if the layout exists
                // TODO: Filter unassigned
                { for tabls.map(|entry_id| self.render_item(entry_id, ctx)) }
            </nav>
        }
    }
}

impl TabSelectorWidget {
    fn render_item(&self, entry_id: EntryId, ctx: &Context<Self>) -> Html {
        let state = self.paths.read();
        let caption = entry_id.to_string();
        let layout = Some(entry_id);
        let selected = layout == state.selected_layout;
        let (class, event) = {
            if selected {
                ("nav-link link-primary active", None)
            } else {
                ("nav-link link-secondary", layout)
            }
        };
        html! {
            <div class="nav-item click">
                <a class=class
                    onclick=ctx.event(Msg::SelectTab(event))
                    >{ caption }</a>
            </div>
        }
    }
}

impl NotificationHandler<DataChanged<SceneState>> for TabSelectorWidget {
    fn handle(
        &mut self,
        _event: DataChanged<SceneState>,
        ctx: &mut Context<Self>,
    ) -> Result<(), Error> {
        ctx.redraw();
        Ok(())
    }
}
