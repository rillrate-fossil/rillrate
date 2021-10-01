use crate::cards::render;
use rill_protocol::io::provider::{EntryId, Path};
use rrpack_basis::manifest::description::PackFlowDescription;
use std::cmp::{Ord, Ordering};
use yew::{html, Html, NodeRef};

pub struct Record {
    pub name: EntryId,
    pub path: Path,
    pub node_ref: NodeRef,
    pub rule: render::RenderRule,
}

impl Record {
    pub fn render(&self) -> Html {
        let record = self;
        let style = "";
        let inner_html = record.rule.render.render(&record.path);
        html! {
            <div class="center" style=style ref=record.node_ref.clone()>
                { inner_html }
            </div>
        }
    }
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
        Some(self.cmp(other))
    }
}

impl PartialEq for Record {
    fn eq(&self, other: &Self) -> bool {
        self.name == other.name
    }
}

impl Eq for Record {}
