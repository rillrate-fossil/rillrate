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
    pub path: AutoPath,
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
        let path = config.path.into();
        Self {
            position,
            size,
            path,
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
    pub path: Path,
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
