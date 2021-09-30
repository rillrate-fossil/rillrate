use super::record::{LabelRecord, Record};
use crate::explorer::state::PATHS;
use rate_ui::shared_object::SharedObject;
use rate_ui::widget::{Context, Widget, WidgetRuntime};
use rrpack_basis::manifest::layouts::layout::Layout;
use yew::{html, Html, Properties};

pub type LayoutViewer = WidgetRuntime<LayoutViewerWidget>;

#[derive(Debug, Properties, Clone, PartialEq, Eq)]
pub struct Props {
    pub layout: Layout,
}

#[derive(Default)]
pub struct LayoutViewerWidget {
    /*
blocks: Vec<Record>,
labels: Vec<LabelRecord>,
*/}

impl Widget for LayoutViewerWidget {
    type Event = ();
    type Tag = ();
    type Properties = Props;
    type Meta = ();

    fn init(&mut self, ctx: &mut Context<Self>) {
        self.on_props(ctx);
    }

    fn on_props(&mut self, ctx: &mut Context<Self>) {
        // TODO: DRY! See `group_viewer`
        let paths = PATHS.with(SharedObject::clone);
        let paths = paths.read();
        let layout = &ctx.properties().layout;
        /*
        self.blocks.clear();
        for item in &layout.items {
            if let Some(desc) = paths.descs.get(&item.path) {
                let record = Record::from((desc, item));
                self.blocks.push(record);
            }
        }
        self.labels.clear();
        for label in &layout.labels {
            let record = LabelRecord::from(label.clone());
            self.labels.push(record);
        }
        */
        ctx.redraw();
    }

    fn view(&self, _ctx: &Context<Self>) -> Html {
        html! {}
    }

    /*
    fn view(&self, _ctx: &Context<Self>) -> Html {
        html! {
            <div yew=module_path!() style="position: relative; width: 100%; height: 100%;">
                { for self.blocks.iter().map(Record::render) }
                { for self.labels.iter().map(LabelRecord::render) }
            </div>
        }
    }
    */
}

fn layout_to_html(layout: &Layout) -> Html {
    html! {}
}
