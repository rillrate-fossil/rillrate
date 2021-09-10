use crate::auto_path::AutoPath;
use rill_protocol::io::provider::{EntryId, Path};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct LayoutConfig {
    pub name: Option<String>,
    pub item: Option<Vec<LayoutItemConfig>>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct LayoutItemConfig {
    // TODO: Use `Position` in non-config
    pub position: (u32, u32),
    pub size: (u32, u32),
    pub path: Option<AutoPath>,
    pub text: Option<String>,
}

impl From<LayoutConfig> for Layout {
    fn from(config: LayoutConfig) -> Self {
        let items = config
            .item
            .unwrap_or_default()
            .into_iter()
            .map(LayoutItem::from)
            .collect();
        Self {
            name: config.name.unwrap_or_default().into(),
            items,
        }
    }
}

impl From<LayoutItemConfig> for LayoutItem {
    fn from(config: LayoutItemConfig) -> Self {
        let position = Position {
            left: config.position.0,
            top: config.position.1,
        };
        let size = Size {
            width: config.size.0,
            height: config.size.1,
        };
        let item_type = (config.path, config.text).into();
        Self {
            position,
            size,
            item_type,
        }
    }
}

impl From<(Option<AutoPath>, Option<String>)> for LayoutItemType {
    fn from(pair: (Option<AutoPath>, Option<String>)) -> Self {
        match pair {
            (Some(path), _) => Self::Flow { path: path.into() },
            (_, Some(text)) => Self::Label { text },
            (None, None) => Self::Label {
                text: "<label>".into(),
            },
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct Layout {
    pub name: EntryId,
    pub items: Vec<LayoutItem>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct LayoutItem {
    pub position: Position,
    pub size: Size,
    pub item_type: LayoutItemType,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum LayoutItemType {
    Flow { path: Path },
    Label { text: String },
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
