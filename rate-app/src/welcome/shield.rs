use crate::cases::state::{GlobalScene, SceneState, SCENE};
use crate::explorer::state::{DashboardState, PATHS};
use anyhow::Error;
use rate_ui::shared_object::{DataChanged, SharedObject};
use rate_ui::widget::{Context, NotificationHandler, Widget, WidgetRuntime};
use rill_protocol::io::provider::EntryId;
use yew::{html, Html};

pub type Shield = WidgetRuntime<ShieldWidget>;

pub struct ShieldWidget {
    scene: SharedObject<SceneState>,
    paths: SharedObject<DashboardState>,
}

impl Default for ShieldWidget {
    fn default() -> Self {
        Self {
            scene: SCENE.with(SharedObject::clone),
            paths: PATHS.with(SharedObject::clone),
        }
    }
}

#[derive(Clone)]
pub enum Msg {
    ChangeScene(GlobalScene),
    ToLayout(EntryId),
    ToDashboard(EntryId, EntryId),
}

impl Widget for ShieldWidget {
    type Event = Msg;
    type Tag = ();
    type Properties = ();
    type Meta = ();

    fn init(&mut self, ctx: &mut Context<Self>) {
        self.scene.subscribe(ctx);
        self.paths.subscribe(ctx);
    }

    fn on_event(&mut self, event: Self::Event, _ctx: &mut Context<Self>) {
        match event {
            Msg::ChangeScene(scene) => {
                self.scene.write().global_scene = scene;
            }
            Msg::ToLayout(entry_id) => {
                let mut scene = self.scene.write();
                scene.global_scene = GlobalScene::Cases;
                // TODO: Use separate route state for seleciton
                scene.selected_layout = Some(entry_id);
            }
            Msg::ToDashboard(package, entry_id) => {
                let mut scene = self.scene.write();
                scene.global_scene = GlobalScene::Explorer;
                let mut paths = self.paths.write();
                paths.selection.selected_package = Some(package);
                paths.selection.selected_dashboard = Some(entry_id);
            }
        }
    }

    fn view(&self, ctx: &Context<Self>) -> Html {
        let scene_state = self.scene.read();
        let paths_state = self.paths.read();
        let to_layouts = ctx.event(Msg::ChangeScene(GlobalScene::Cases));
        let to_explorer = ctx.event(Msg::ChangeScene(GlobalScene::Explorer));
        html! {
            <div class="flex-grow-1">
                <div class="container mt-5">
                    <h2 class="fw-bold"><span class="text-primary">{ "Live" }</span>{ " Dashboard" }</h2>
                    <h3 class="mt-4 mb-3 pointer" onclick=to_layouts>{ "Cases" }</h3>
                    <div class="d-flex flex-row flex-wrap">
                        { for scene_state.layouts.keys().map(|entry| self.render_layout_card(entry, ctx)) }
                    </div>
                    <h3 class="mt-4 mb-3 pointer" onclick=to_explorer>{ "Explorer" }</h3>
                    { for paths_state.structure.get_packages().map(|entry| self.render_package(entry, ctx)) }
                </div>
            </div>
        }
    }
}

const CARD_CLASS: &str = "card me-3 mb-3 bg-primary text-white shadow-sm";
const CARD_BUTTON: &str = "btn btn-outline-primary stretched-link";

impl ShieldWidget {
    fn render_package(&self, entry_id: &EntryId, ctx: &Context<Self>) -> Html {
        let paths_state = self.paths.read();
        html! {
            <div>
                <h4>{ entry_id }</h4>
                <div class="d-flex flex-row flex-wrap">
                    { for paths_state.structure.get_dashboards(entry_id).map(|entry| self.render_dashboard_card(entry_id, entry, ctx)) }
                </div>
            </div>
        }
    }

    fn render_layout_card(&self, entry_id: &EntryId, ctx: &Context<Self>) -> Html {
        let callback = ctx.event(Msg::ToLayout(entry_id.clone()));
        html! {
            <div class=CARD_CLASS>
                <div class="card-body text-center" style="width: 16rem;">
                    <h5 class="card-title mb-3">{ entry_id }</h5>
                    <a class=CARD_BUTTON onclick=callback>{ "Open" }</a>
                </div>
            </div>
        }
    }

    fn render_dashboard_card(
        &self,
        package: &EntryId,
        entry_id: &EntryId,
        ctx: &Context<Self>,
    ) -> Html {
        let callback = ctx.event(Msg::ToDashboard(package.clone(), entry_id.clone()));
        html! {
            <div class=CARD_CLASS>
                <div class="card-body text-center" style="width: 16rem;">
                    <h5 class="card-title mb-3">{ entry_id }</h5>
                    <a class=CARD_BUTTON onclick=callback>{ "Open" }</a>
                </div>
            </div>
        }
    }
}

impl NotificationHandler<DataChanged<SceneState>> for ShieldWidget {
    fn handle(
        &mut self,
        _event: DataChanged<SceneState>,
        ctx: &mut Context<Self>,
    ) -> Result<(), Error> {
        ctx.redraw();
        Ok(())
    }
}

impl NotificationHandler<DataChanged<DashboardState>> for ShieldWidget {
    fn handle(
        &mut self,
        _event: DataChanged<DashboardState>,
        ctx: &mut Context<Self>,
    ) -> Result<(), Error> {
        ctx.redraw();
        Ok(())
    }
}
