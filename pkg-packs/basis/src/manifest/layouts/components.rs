use crate::paths::AutoPath;
use derive_more::From;
use ordered_float::OrderedFloat;
use rill_protocol::io::provider::Path;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct Layout {
    pub name: Path,
    pub element: Element,
}

pub type BoxedElement = Box<Element>;

pub trait Boxed {
    fn boxed(self) -> BoxedElement;
}

impl<T: Into<Element>> Boxed for T {
    fn boxed(self) -> BoxedElement {
        Box::new(self.into())
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, From)]
pub enum Element {
    Empty,

    // Containers
    Align(Align),
    Center(Center),
    Container(Container),
    Expanded(Expanded),
    Spacer(Spacer),
    Row(Row),
    Column(Column),

    // Components
    Text(Text),
    Flow(Flow),
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, From)]
pub struct Align {
    pub alignment: Alignment,
    pub child: BoxedElement,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, From)]
pub struct Center {
    pub child: BoxedElement,
}

impl Center {
    pub fn new(child: impl Into<Element>) -> Self {
        Self {
            child: child.boxed(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, From)]
pub struct Container {
    pub child: BoxedElement,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, From)]
pub struct Expanded {
    pub child: BoxedElement,
    pub flex: OrderedFloat<f64>,
}

impl Expanded {
    pub fn new(child: impl Into<Element>, flex: f64) -> Self {
        Self {
            child: child.boxed(),
            flex: OrderedFloat(flex),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, From)]
pub struct Spacer {
    pub flex: OrderedFloat<f64>,
    pub maintenance: bool,
}

impl Spacer {
    pub fn new(flex: f64) -> Self {
        Self {
            flex: OrderedFloat(flex),
            maintenance: false,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, From)]
pub struct Row {
    pub children: Vec<Element>,
}

impl Row {
    pub fn new(children: Vec<Element>) -> Self {
        Self { children }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, From)]
pub struct Column {
    pub children: Vec<Element>,
}

impl Column {
    pub fn new(children: Vec<Element>) -> Self {
        Self { children }
    }
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
pub struct Text {
    pub text: String,
    pub align: TextAlign,
}

impl Text {
    pub fn new(text: impl Into<String>, align: TextAlign) -> Self {
        Self {
            text: text.into(),
            align,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, From)]
pub enum TextAlign {
    Left = 0,
    Right = 1,
    Center = 2,
    Justify = 3,
    Start = 4,
    End = 5,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, From)]
pub struct Flow {
    pub path: Path,
}

impl Flow {
    pub fn new(path: impl Into<AutoPath>) -> Self {
        Self {
            path: path.into().into(),
        }
    }
}
