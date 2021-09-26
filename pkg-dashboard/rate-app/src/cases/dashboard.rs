use super::state::{CasesState, CASES};
use crate::welcome::state::{GlobalScene, SceneState, SCENE};
use anyhow::Error;
use rate_ui::shared_object::{DataChanged, SharedObject};
use rate_ui::widget::{Context, NotificationHandler, Widget, WidgetRuntime};
use yew::{html, Html};

pub type Dashboard = WidgetRuntime<DashboardWidget>;

pub struct DashboardWidget {
    scene: SharedObject<SceneState>,
    cases: SharedObject<CasesState>,
}

impl Default for DashboardWidget {
    fn default() -> Self {
        Self {
            scene: SCENE.with(SharedObject::clone),
            cases: CASES.with(SharedObject::clone),
        }
    }
}

#[derive(Clone)]
pub enum Msg {
    ToExplorer,
}

impl Widget for DashboardWidget {
    type Event = Msg;
    type Tag = ();
    type Properties = ();
    type Meta = ();

    fn init(&mut self, ctx: &mut Context<Self>) {
        self.cases.subscribe(ctx);
    }

    fn on_event(&mut self, event: Self::Event, _ctx: &mut Context<Self>) {
        match event {
            Msg::ToExplorer => {
                let mut scene = self.scene.write();
                scene.global_scene = GlobalScene::Explorer;
            }
        }
    }

    fn view(&self, ctx: &Context<Self>) -> Html {
        let state = self.cases.read();
        if let Some(layout) = state.get_layout_tab() {
            html! {
                <super::LayoutViewer layout_tab=layout.clone() />
            }
        } else {
            html! {
                <div>
                    <p>
                        { "No cases available. Try to use: " }
                        <a class="link click" onclick=ctx.event(Msg::ToExplorer)>{ "Explorer" }</a>
                    </p>
                </div>
            }
        }
    }
}

impl NotificationHandler<DataChanged<CasesState>> for DashboardWidget {
    fn handle(
        &mut self,
        _event: DataChanged<CasesState>,
        ctx: &mut Context<Self>,
    ) -> Result<(), Error> {
        ctx.redraw();
        Ok(())
    }
}
