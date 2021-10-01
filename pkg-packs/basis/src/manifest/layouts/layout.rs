use serde::{Deserialize, Serialize};

/*
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
*/

// TODO: Consider moving to the UI
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
