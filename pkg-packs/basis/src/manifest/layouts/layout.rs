use super::components::Container;
use rill_protocol::io::provider::Path;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct Layout {
    #[serde(deserialize_with = "super::unpack::from_str")]
    pub name: Path,
    #[serde(rename = "$value")]
    pub container: Container,
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
