use rate_config::{Config, ReadableConfig};
use rrpack_basis::auto_path::AutoPath;
use rrpack_basis::manifest::layouts::layout::{Label, Layout, LayoutItem, LayoutTab};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct CaseConfig {
    pub name: String,
    pub tab: Vec<CaseTabConfig>,
}

impl Config for CaseConfig {}

impl ReadableConfig for CaseConfig {}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct CaseTabConfig {
    pub name: String,
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

impl From<CaseConfig> for Layout {
    fn from(config: CaseConfig) -> Self {
        let tabs = config.tab.into_iter().map(LayoutTab::from).collect();
        Self {
            name: config.name.into(),
            tabs,
        }
    }
}

impl From<CaseTabConfig> for LayoutTab {
    fn from(config: CaseTabConfig) -> Self {
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
            name: config.name.into(),
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
