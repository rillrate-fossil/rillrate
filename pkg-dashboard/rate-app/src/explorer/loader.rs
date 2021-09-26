use super::state::{ExplorerState, ExplorerStructure, PATHS};
use rate_ui::shared_object::SharedObject;
use rate_ui::widget::wired_widget::{SingleFlowMeta, WiredWidget};
use rate_ui::widget::{Context, Widget, WidgetRuntime};
use rill_protocol::io::provider::Path;
use rrpack_basis::manifest::paths::{PathsSpec, PathsState};
use yew::Html;

pub type Loader = WidgetRuntime<LoaderWidget>;

pub struct LoaderWidget {
    paths: SharedObject<ExplorerState>,
}

impl Default for LoaderWidget {
    fn default() -> Self {
        Self {
            paths: PATHS.with(SharedObject::clone),
        }
    }
}

impl Widget for LoaderWidget {
    type Event = ();
    type Tag = Option<Path>;
    type Properties = ();
    type Meta = SingleFlowMeta<Self>;

    fn init(&mut self, ctx: &mut Context<Self>) {
        let path = PathsSpec::path().of_server();
        ctx.rewire(path);
    }

    fn view(&self, _ctx: &Context<Self>) -> Html {
        Html::default()
    }
}

impl WiredWidget<SingleFlowMeta<Self>> for LoaderWidget {
    type Flow = PathsState;

    fn state_changed(&mut self, _reloaded: bool, ctx: &mut Context<Self>) {
        // TODO: Consider to process deltas instead!
        let mut new_structure = ExplorerStructure::default();
        if let Some(state) = ctx.meta().state() {
            //log::error!("DASHBOARD: {:?}", state);
            for path in state.records.keys().cloned() {
                let mut items = path.into_iter();

                let packages = &mut new_structure.packages;
                if let Some(package) = items.next() {
                    let dashboards = packages.entry(package).or_default();
                    if let Some(dashboard) = items.next() {
                        let groups = dashboards.entry(dashboard).or_default();
                        if let Some(group) = items.next() {
                            let streams = groups.entry(group).or_default();
                            if let Some(stream) = items.next() {
                                streams.insert(stream);
                            }
                        }
                    }
                }
            }
            let mut paths = self.paths.write();
            paths.structure = new_structure;
            // TODO: Avoid cloning here!!!
            paths.descs = state.records.clone();
        }
    }
}
