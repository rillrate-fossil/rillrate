use crate::cards::render;
use rate_ui::packages::talent::flexlayout::Item;
use rill_protocol::io::provider::{EntryId, Path};
use rrpack_prime::manifest::description::PackFlowDescription;
use std::cmp::{Ord, Ordering};
use yew::{html, Html, NodeRef};

impl Record {
    pub fn render(item: Option<&Item<Self>>) -> Html {
        if let Some(item) = item {
            let record = &item.record;
            let inner_html = (record.rule.render)(&record.path);
            let style = format!("order: {};", item.order,);
            let size = format!(
                " width: {}px; height: {}px;",
                record.rule.size.width, record.rule.size.height,
            );
            html! {
                <div class="pe-3 pb-3 d-flex" style=style ref=record.node_ref.clone()>
                    <div class="bg-light shadow-sm w-100 d-flex flex-column">
                        <div class="pt-3 text-center caption">{ &record.name }</div>
                        <div style=size>
                            { inner_html }
                        </div>
                    </div>
                </div>
            }
        } else {
            html! {
                <div class="invisible">
                </div>
            }
        }
    }
}

pub struct Record {
    pub name: EntryId,
    pub path: Path,
    pub node_ref: NodeRef,
    pub rule: render::RenderRule,
}

impl From<&PackFlowDescription> for Record {
    fn from(desc: &PackFlowDescription) -> Self {
        let rule = render::RENDERS
            .get(&desc.stream_type)
            .unwrap_or(&render::RENDER_DEFAULT)
            .clone();
        Self {
            name: desc.path.last().cloned().unwrap_or_default(),
            path: desc.path.clone(),
            node_ref: NodeRef::default(),
            rule,
        }
    }
}

impl Ord for Record {
    fn cmp(&self, other: &Self) -> Ordering {
        self.name.cmp(&other.name)
    }
}

impl PartialOrd for Record {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(&other))
    }
}

impl PartialEq for Record {
    fn eq(&self, other: &Self) -> bool {
        self.name == other.name
    }
}

impl Eq for Record {}
