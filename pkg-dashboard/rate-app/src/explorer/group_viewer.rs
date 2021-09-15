use super::record::Record;
use super::state::{ResolvedGroup, PATHS};
use rate_ui::packages::talent::flexlayout::FlexLayout;
use rate_ui::shared_object::SharedObject;
use rate_ui::widget::{Context, Widget, WidgetRuntime};
use rill_protocol::diff::diff;
use rill_protocol::io::provider::Path;
use yew::{html, Html, Properties};

pub type GroupViewer = WidgetRuntime<GroupViewerWidget>;

#[derive(Debug, Properties, Clone, PartialEq, Eq)]
pub struct Props {
    pub streams: ResolvedGroup,
}

#[derive(Default)]
pub struct GroupViewerWidget {
    layout: FlexLayout<Path, Record>,
}

impl Widget for GroupViewerWidget {
    type Event = ();
    type Tag = ();
    type Properties = Props;
    type Meta = ();

    fn init(&mut self, ctx: &mut Context<Self>) {
        self.on_props(ctx);
    }

    fn on_props(&mut self, ctx: &mut Context<Self>) {
        // TODO: DRY! See `layout_viewer`
        let paths = PATHS.with(SharedObject::clone);
        let paths = paths.read();
        let descs = &ctx.properties().streams;
        let (to_add, to_remove) = diff(self.layout.keys(), descs.keys());
        for path in to_add {
            if let Some(desc) = paths.descs.get(&path) {
                let record = Record::from(desc);
                self.layout.acquire(path, record);
            }
        }
        for name in to_remove {
            self.layout.release(name);
        }
        if ctx.is_rendered() {
            ctx.redraw();
        }
    }

    fn view(&self, _ctx: &Context<Self>) -> Html {
        html! {
            <div yew=module_path!() class="d-flex flex-row flex-wrap">
                { for self.layout.values().map(Record::render) }
            </div>
        }
    }
}
