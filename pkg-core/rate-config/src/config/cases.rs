use rill_config::{Config, ReadableConfig};
use rill_protocol::io::provider::{EntryId, Path};
use rrpack_basis::manifest::layouts::layout::{Label, LayoutItem, LayoutTab};
use rrpack_basis::paths::AutoPath;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct CaseConfig {
    /// It used to add `group` name prefix
    name: EntryId,
    pub tab: Option<Vec<CaseTabConfig>>,
}

impl Config for CaseConfig {}

impl ReadableConfig for CaseConfig {}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct CaseTabConfig {
    pub name: EntryId,
    pub item: Option<Vec<CaseItemConfig>>,
    pub label: Option<Vec<LabelConfig>>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct CaseItemConfig {
    pub position: (i32, i32),
    pub size: (i32, i32),
    pub path: AutoPath,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct LabelConfig {
    pub position: (i32, i32),
    pub size: (i32, i32),
    pub text: String,
}

impl CaseConfig {
    pub fn tabs(self) -> impl Iterator<Item = LayoutTab> {
        let group = self.name;
        self.tab
            .into_iter()
            .flatten()
            .map(move |tab| CaseTabConfigPair::new(group.clone(), tab))
            .map(LayoutTab::from)
    }
}

pub struct CaseTabConfigPair {
    pub path: Path,
    pub config: CaseTabConfig,
}

impl CaseTabConfigPair {
    fn new(group: EntryId, config: CaseTabConfig) -> Self {
        Self {
            path: [group, config.name.clone()].to_vec().into(),
            config,
        }
    }
}

impl From<CaseTabConfigPair> for LayoutTab {
    fn from(pair: CaseTabConfigPair) -> Self {
        let CaseTabConfigPair { path, config } = pair;
        let items = config
            .item
            .unwrap_or_default()
            .into_iter()
            .map(LayoutItem::from)
            .collect();
        let labels = config
            .label
            .unwrap_or_default()
            .into_iter()
            .map(Label::from)
            .collect();
        Self {
            name: path,
            items,
            labels,
        }
    }
}

impl From<CaseItemConfig> for LayoutItem {
    fn from(config: CaseItemConfig) -> Self {
        Self {
            position: config.position.into(),
            size: config.size.into(),
            path: config.path.into(),
        }
    }
}

impl From<LabelConfig> for Label {
    fn from(config: LabelConfig) -> Self {
        Self {
            position: config.position.into(),
            size: config.size.into(),
            text: config.text,
        }
    }
}
