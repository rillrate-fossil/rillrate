use super::state::{CasesState, CasesStructure, CASES};
use rate_ui::shared_object::SharedObject;
use rate_ui::widget::wired_widget::{SingleFlowMeta, WiredWidget};
use rate_ui::widget::{Context, Widget, WidgetRuntime};
use rill_protocol::io::provider::Path;
use rrpack_basis::manifest::layouts::{LayoutsSpec, LayoutsState};
use yew::Html;

pub type Loader = WidgetRuntime<LoaderWidget>;

pub struct LoaderWidget {
    cases: SharedObject<CasesState>,
}

impl Default for LoaderWidget {
    fn default() -> Self {
        Self {
            cases: CASES.with(SharedObject::clone),
        }
    }
}

impl Widget for LoaderWidget {
    type Event = ();
    type Tag = Option<Path>;
    type Properties = ();
    type Meta = SingleFlowMeta<Self>;

    fn init(&mut self, ctx: &mut Context<Self>) {
        let path = LayoutsSpec::path().of_server();
        ctx.rewire(path);
    }

    fn view(&self, _ctx: &Context<Self>) -> Html {
        Html::default()
    }
}

impl WiredWidget<SingleFlowMeta<Self>> for LoaderWidget {
    type Flow = LayoutsState;

    fn state_changed(&mut self, _reloaded: bool, ctx: &mut Context<Self>) {
        // Apply change to the router state
        // TODO: Use `CasesStructure` instead
        let mut new_structure = CasesStructure::default();
        if let Some(state) = ctx.meta().state() {
            let layouts = &mut new_structure.layouts;
            for path in state.layouts.keys().cloned() {
                let mut items = path.into_iter();
                if let Some(layout) = items.next() {
                    let tabs = layouts.entry(layout).or_default();
                    if let Some(tab) = items.next() {
                        tabs.insert(tab);
                    }
                }
            }
            let mut cases = self.cases.write();
            cases.structure = new_structure;
            // TODO: Avoid cloning here!!!
            cases.tabs = state.layouts.clone();
        }
    }
}
