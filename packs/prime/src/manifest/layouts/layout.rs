use crate::auto_path::AutoPath;
use rill_protocol::io::provider::{EntryId, Path};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct LayoutConfig {
    pub name: Option<String>,
    pub item: Option<Vec<LayoutItemConfig>>,
    pub label: Option<Vec<LabelConfig>>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct LayoutItemConfig {
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

impl From<LayoutConfig> for Layout {
    fn from(config: LayoutConfig) -> Self {
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

impl From<LayoutItemConfig> for LayoutItem {
    fn from(config: LayoutItemConfig) -> Self {
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

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct Layout {
    pub name: EntryId,
    pub items: Vec<LayoutItem>,
    pub labels: Vec<Label>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct LayoutItem {
    pub position: Position,
    pub size: Size,
    pub path: Path,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct Label {
    pub position: Position,
    pub size: Size,
    pub text: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct Position {
    pub left: u32,
    pub top: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct Size {
    pub width: u32,
    pub height: u32,
}
