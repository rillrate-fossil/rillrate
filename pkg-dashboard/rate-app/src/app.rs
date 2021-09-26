use crate::welcome::state::{GlobalScene, SceneState, SCENE};
use crate::{alerts, cases, explorer, welcome};
use anyhow::Error;
use rate_ui::agents::live::LiveAgent;
use rate_ui::shared_object::{DataChanged, SharedObject};
use rate_ui::widget::{Context, NotificationHandler, OnWireEvent, Widget, WidgetRuntime};
use yew::{html, Html};

pub type App = WidgetRuntime<AppWidget>;

pub struct AppWidget {
    scene: SharedObject<SceneState>,
}

impl Default for AppWidget {
    fn default() -> Self {
        Self {
            scene: SCENE.with(SharedObject::clone),
        }
    }
}

impl Widget for AppWidget {
    type Event = ();
    type Tag = ();
    type Properties = ();
    type Meta = ();

    fn init(&mut self, ctx: &mut Context<Self>) {
        // Just to keep the connection alive all the time.
        ctx.live();
        self.scene.subscribe(ctx);
    }

    fn view(&self, _ctx: &Context<Self>) -> Html {
        let state = self.scene.read();
        let menu;
        let body;
        match &state.global_scene {
            GlobalScene::Home => {
                menu = html! {};
                body = html! {
                    <welcome::Shield />
                };
            }
            GlobalScene::Cases => {
                menu = html! {
                    <cases::DashboardMenu />
                };
                body = html! {
                    <cases::Dashboard />
                };
            }
            GlobalScene::Explorer => {
                menu = html! {
                    <explorer::DashboardMenu />
                };
                body = html! {
                    <explorer::Dashboard />
                };
            }
        }
        html! {
            <div class="d-flex flex-column bg-white" style="min-height: 100vh;">
                <nav class="navbar z-1000 border shadow-sm bg-light">
                    <div class="container-fluid">
                        <div class="navbar-brand me-3">
                            <div class="fw-bold">{ "RillRate" }</div>
                        </div>
                        <div class="mx-2">
                            <welcome::SceneSelector />
                        </div>
                        <div class="flex-grow-1 d-flex flex-row justify-content-between">
                            { menu }
                        </div>
                    </div>
                </nav>
                <div class="flex-grow-1 p-3" style="width: 100%; height: 100%;">
                    { body }
                </div>
                <nav class="navbar bg-light" style="height: 50px;">
                    <div class="container-fluid">
                        <div class="text-secondary">{ "© 2021 RillRate OÜ" }</div>
                        <div>{ "v" }{ crate::meta::VERSION }</div>
                        <img style="height: 1.8rem;" src="./images/logo.svg" />
                    </div>
                </nav>
                <alerts::AlertToast />
                <cases::Loader />
                <explorer::Loader />
            </div>
        }
    }
}

impl OnWireEvent<LiveAgent> for AppWidget {}

impl NotificationHandler<DataChanged<SceneState>> for AppWidget {
    fn handle(
        &mut self,
        _event: DataChanged<SceneState>,
        ctx: &mut Context<Self>,
    ) -> Result<(), Error> {
        ctx.redraw();
        Ok(())
    }
}
