use rill_protocol::io::provider::{EntryId, Path};
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct Layout {
    pub name: EntryId,
    pub tabs: BTreeMap<EntryId, LayoutTab>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct LayoutTab {
    pub name: EntryId,
    pub items: Vec<LayoutItem>,
    pub labels: Vec<Label>,
}

#[cfg(feature = "engine")]
pub mod layout_builder {
    use super::*;
    use crate::manifest::layouts::global::LAYOUTS;
    use crate::AutoPath;

    // TODO: Consider to add the separate `LayoutBuilder` struct
    impl Layout {
        pub fn new(name: impl Into<EntryId>) -> Self {
            Self {
                name: name.into(),
                tabs: BTreeMap::new(),
            }
        }

        pub fn add_tab(&mut self, tab: LayoutTab) {
            self.tabs.insert(tab.name.clone(), tab);
        }

        pub fn register(&self) {
            let name = self.name.clone();
            let layout = self.clone();
            LAYOUTS.add_layout(name, layout);
        }

        pub fn unregister(&self) {
            let name = self.name.clone();
            LAYOUTS.remove_layout(name);
        }
    }

    impl LayoutTab {
        pub fn new(name: impl Into<EntryId>) -> Self {
            Self {
                name: name.into(),
                items: Vec::new(),
                labels: Vec::new(),
            }
        }

        pub fn add_item(
            &mut self,
            position: impl Into<Position>,
            size: impl Into<Size>,
            path: impl Into<AutoPath>,
        ) {
            let item = LayoutItem {
                position: position.into(),
                size: size.into(),
                path: Path::from(path.into()),
            };
            self.items.push(item);
        }
    }
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
    pub left: i32,
    pub top: i32,
}

impl<L, T> From<(L, T)> for Position
where
    L: Into<i32>,
    T: Into<i32>,
{
    fn from((left, top): (L, T)) -> Self {
        Self {
            left: left.into(),
            top: top.into(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct Size {
    pub width: i32,
    pub height: i32,
}

impl<W, H> From<(W, H)> for Size
where
    W: Into<i32>,
    H: Into<i32>,
{
    fn from((width, height): (W, H)) -> Self {
        Self {
            width: width.into(),
            height: height.into(),
        }
    }
}
