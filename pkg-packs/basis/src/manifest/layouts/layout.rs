use derive_more::From;
use ordered_float::OrderedFloat;
use rill_protocol::io::provider::Path;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, From)]
pub enum Container {
    Empty,
    Align(Align),
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, From)]
pub struct Align {
    pub alignment: Alignment,
    pub child: Element,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
pub struct Alignment {
    pub x: OrderedFloat<f64>,
    pub y: OrderedFloat<f64>,
}

impl Alignment {
    pub const fn new(x: f64, y: f64) -> Self {
        Self {
            x: OrderedFloat(x),
            y: OrderedFloat(y),
        }
    }
}

impl Alignment {
    pub const BOTTOM_CENTER: Self = Self::new(0.0, 1.0);
    pub const BOTTOM_LEFT: Self = Self::new(-1.0, 1.0);
    pub const BOTTOM_RIGHT: Self = Self::new(1.0, 1.0);
    pub const CENTER: Self = Self::new(0.0, 0.0);
    pub const CENTER_LEFT: Self = Self::new(-1.0, 0.0);
    pub const CENTER_RIGHT: Self = Self::new(1.0, 0.0);
    pub const TOP_CENTER: Self = Self::new(0.0, -1.0);
    pub const TOP_LEFT: Self = Self::new(-1.0, -1.0);
    pub const TOP_RIGHT: Self = Self::new(1.0, -1.0);
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, From)]
pub enum Element {
    //Container { container: Container },
    Label(LLabel),
    //Flow { path: Path },
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, From)]
pub struct LLabel {
    pub text: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct Layout {
    pub name: Path,
    pub container: Container,
    pub items: Vec<LayoutItem>,
    pub labels: Vec<Label>,
}

#[cfg(feature = "engine")]
pub mod layout_builder {
    use super::*;
    use crate::manifest::layouts::global::LAYOUTS;
    use crate::paths::{AutoPath, LayoutPath};

    impl Layout {
        pub fn new(name: impl Into<LayoutPath>) -> Self {
            Self {
                name: name.into().into(),
                container: Container::Empty,
                items: Vec::new(),
                labels: Vec::new(),
            }
        }

        pub fn set_container(&mut self, container: impl Into<Container>) {
            self.container = container.into();
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

        pub fn add_label(
            &mut self,
            position: impl Into<Position>,
            size: impl Into<Size>,
            text: impl Into<String>,
        ) {
            let item = Label {
                position: position.into(),
                size: size.into(),
                text: text.into(),
            };
            self.labels.push(item);
        }

        pub fn register(&self) {
            let name = self.name.clone();
            let layout = self.clone();
            LAYOUTS.add_tab(name, layout);
        }

        pub fn unregister(&self) {
            let name = self.name.clone();
            LAYOUTS.remove_tab(name);
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
