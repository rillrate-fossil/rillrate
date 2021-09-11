use rate_config::{Config, ReadableConfig};
use rrpack_prime::auto_path::AutoPath;
use rrpack_prime::manifest::layouts::layout::{Label, Layout, LayoutItem, Position, Size};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct CaseConfig {
    pub name: Option<String>,
    pub item: Option<Vec<CaseItemConfig>>,
    pub label: Option<Vec<LabelConfig>>,
}

impl Config for CaseConfig {}

impl ReadableConfig for CaseConfig {}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct CaseItemConfig {
    pub position: (u32, u32),
    pub size: (u32, u32),
    pub path: AutoPath,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct LabelConfig {
    pub position: (u32, u32),
    pub size: (u32, u32),
    pub text: String,
}

impl From<CaseConfig> for Layout {
    fn from(config: CaseConfig) -> Self {
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
            name: config.name.unwrap_or_default().into(),
            items,
            labels,
        }
    }
}

impl From<CaseItemConfig> for LayoutItem {
    fn from(config: CaseItemConfig) -> Self {
        // TODO: DRY
        let position = Position {
            left: config.position.0,
            top: config.position.1,
        };
        let size = Size {
            width: config.size.0,
            height: config.size.1,
        };
        Self {
            position,
            size,
            path: config.path.into(),
        }
    }
}

impl From<LabelConfig> for Label {
    fn from(config: LabelConfig) -> Self {
        // TODO: DRY
        let position = Position {
            left: config.position.0,
            top: config.position.1,
        };
        let size = Size {
            width: config.size.0,
            height: config.size.1,
        };
        Self {
            position,
            size,
            text: config.text,
        }
    }
}
