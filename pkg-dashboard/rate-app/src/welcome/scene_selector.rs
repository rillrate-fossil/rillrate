use crate::welcome::state::{GlobalScene, SceneState, SCENE};
use anyhow::Error;
use rate_ui::shared_object::{DataChanged, SharedObject};
use rate_ui::widget::{Context, NotificationHandler, Widget, WidgetRuntime};
use strum::IntoEnumIterator;
use yew::{html, Html};
use yew_components::Select;

pub type SceneSelector = WidgetRuntime<SceneSelectorWidget>;

pub struct SceneSelectorWidget {
    cases: SharedObject<SceneState>,
}

impl Default for SceneSelectorWidget {
    fn default() -> Self {
        Self {
            cases: SCENE.with(SharedObject::clone),
        }
    }
}

#[derive(Clone)]
pub enum Msg {
    ChangeScene(GlobalScene),
}

impl Widget for SceneSelectorWidget {
    type Event = Msg;
    type Tag = ();
    type Properties = ();
    type Meta = ();

    fn init(&mut self, ctx: &mut Context<Self>) {
        self.cases.subscribe(ctx);
    }

    fn on_event(&mut self, event: Self::Event, _ctx: &mut Context<Self>) {
        match event {
            Msg::ChangeScene(cases) => {
                self.cases.write().global_scene = cases;
            }
        }
    }

    fn view(&self, ctx: &Context<Self>) -> Html {
        let selected = self.cases.read().global_scene.clone();
        let options: Vec<_> = GlobalScene::iter().collect();
        html! {
            <form yew=module_path!() class="d-flex">
                <Select<GlobalScene>
                    class="form-select pointer"
                    options=options
                    selected=selected
                    on_change=ctx.callback(Msg::ChangeScene)
                />
            </form>
        }
    }
}

impl NotificationHandler<DataChanged<SceneState>> for SceneSelectorWidget {
    fn handle(
        &mut self,
        _event: DataChanged<SceneState>,
        ctx: &mut Context<Self>,
    ) -> Result<(), Error> {
        ctx.redraw();
        Ok(())
    }
}
