use crate::cards::render;
use rill_protocol::io::provider::{EntryId, Path};
use rrpack_basis::manifest::description::PackFlowDescription;
use rrpack_basis::manifest::layouts::layout::{Label, LayoutItem, Position, Size};
use std::cmp::{Ord, Ordering};
use yew::{html, Html, NodeRef};

pub struct LabelRecord {
    pub text: String,
    pub node_ref: NodeRef,
    pub position: Position,
    pub size: Size,
}

impl From<Label> for LabelRecord {
    fn from(label: Label) -> Self {
        Self {
            text: label.text,
            node_ref: NodeRef::default(),
            position: label.position,
            size: label.size,
        }
    }
}

impl LabelRecord {
    pub fn render(&self) -> Html {
        let record = self;
        let top = record.position.top;
        let left = record.position.left;
        let height = record.size.height;
        let width = record.size.width;
        let style = format!(
            "position: absolute; top: {}%; left: {}%; height: {}%; width: {}%;",
            top, left, height, width
        );
        html! {
            <div class="fs-3" style=style ref=record.node_ref.clone()>
                { &record.text }
            </div>
        }
    }
}

pub struct Record {
    pub name: EntryId,
    pub path: Path,
    pub node_ref: NodeRef,
    pub rule: render::RenderRule,
    pub position: Position,
    pub size: Size,
}

impl Record {
    pub fn render(&self) -> Html {
        let record = self;
        let top = record.position.top;
        let left = record.position.left;
        let height = record.size.height;
        let width = record.size.width;
        let style = format!(
            "position: absolute; top: {}%; left: {}%; height: {}%; width: {}%;",
            top, left, height, width
        );
        let inner_html = record.rule.render.render(&record.path);
        html! {
            <div style=style ref=record.node_ref.clone()>
                { inner_html }
            </div>
        }
    }
}

impl From<(&PackFlowDescription, &LayoutItem)> for Record {
    fn from((desc, item): (&PackFlowDescription, &LayoutItem)) -> Self {
        let rule = render::RENDERS
            .get(&desc.stream_type)
            .unwrap_or(&render::RENDER_DEFAULT)
            .clone();
        Self {
            name: desc.path.last().cloned().unwrap_or_default(),
            path: desc.path.clone(),
            node_ref: NodeRef::default(),
            rule,
            position: item.position.clone(),
            size: item.size.clone(),
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
