use ordered_float::OrderedFloat;
use rill_protocol::io::provider::Path;
use rrpack_basis::manifest::layouts::components as basis;
use serde::{de::Error, Deserialize, Deserializer};
use std::fmt::Display;
use std::str::FromStr;

pub type BoxedElement = Box<Element>;

impl From<BoxedElement> for basis::BoxedElement {
    fn from(value: BoxedElement) -> Self {
        Box::new(basis::Element::from(*value))
    }
}

#[derive(Debug, Clone, Deserialize, PartialEq, Eq)]
pub struct Layout {
    #[serde(deserialize_with = "from_str")]
    pub name: Path,
    #[serde(rename = "$value")]
    pub element: Element,
}

impl From<Layout> for basis::Layout {
    fn from(value: Layout) -> Self {
        Self {
            name: value.name,
            element: value.element.into(),
        }
    }
}

#[derive(Debug, Clone, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum Element {
    Empty,

    // Containers
    Align(Align),
    Expanded(Expanded),
    Spacer(Spacer),
    Row(Row),
    Column(Column),

    // Components
    Label(Label),
    Flow(Flow),
}

impl From<Element> for basis::Element {
    fn from(value: Element) -> Self {
        match value {
            Element::Empty => Self::Empty,

            // Containers
            Element::Align(value) => Self::Align(value.into()),
            Element::Expanded(value) => Self::Expanded(value.into()),
            Element::Spacer(value) => Self::Spacer(value.into()),
            Element::Row(value) => Self::Row(value.into()),
            Element::Column(value) => Self::Column(value.into()),

            // Components
            Element::Label(value) => Self::Label(value.into()),
            Element::Flow(value) => Self::Flow(value.into()),
        }
    }
}

#[derive(Debug, Clone, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub struct Align {
    pub alignment: Alignment,
    pub child: BoxedElement,
}

impl From<Align> for basis::Align {
    fn from(value: Align) -> Self {
        Self {
            alignment: value.alignment.into(),
            child: Box::new(basis::Element::from(*value.child)),
        }
    }
}

#[derive(Debug, Clone, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub struct Expanded {
    pub child: BoxedElement,
    pub flex: OrderedFloat<f64>,
}

impl From<Expanded> for basis::Expanded {
    fn from(value: Expanded) -> Self {
        Self {
            child: value.child.into(),
            flex: value.flex,
        }
    }
}

#[derive(Debug, Clone, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub struct Spacer {
    pub flex: OrderedFloat<f64>,
}

impl From<Spacer> for basis::Spacer {
    fn from(value: Spacer) -> Self {
        Self { flex: value.flex }
    }
}

#[derive(Debug, Clone, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub struct Row {
    #[serde(rename = "$value")]
    pub children: Vec<Element>,
}

impl From<Row> for basis::Row {
    fn from(value: Row) -> Self {
        Self {
            children: value
                .children
                .into_iter()
                .map(From::<Element>::from)
                .collect(),
        }
    }
}

#[derive(Debug, Clone, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub struct Column {
    #[serde(rename = "$value")]
    pub children: Vec<Element>,
}

impl From<Column> for basis::Column {
    fn from(value: Column) -> Self {
        Self {
            children: value
                .children
                .into_iter()
                .map(From::<Element>::from)
                .collect(),
        }
    }
}

#[derive(Debug, Clone, Copy, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub struct Alignment {
    pub x: OrderedFloat<f64>,
    pub y: OrderedFloat<f64>,
}

impl From<Alignment> for basis::Alignment {
    fn from(value: Alignment) -> Self {
        Self {
            x: value.x,
            y: value.y,
        }
    }
}

#[derive(Debug, Clone, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub struct Label {
    #[serde(rename = "$value")]
    pub text: String,
}

impl From<Label> for basis::Label {
    fn from(value: Label) -> Self {
        Self { text: value.text }
    }
}

#[derive(Debug, Clone, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub struct Flow {
    #[serde(deserialize_with = "from_str")]
    pub path: Path,
}

impl From<Flow> for basis::Flow {
    fn from(value: Flow) -> Self {
        Self { path: value.path }
    }
}

pub fn from_str<'de, T, D>(deserializer: D) -> Result<T, D::Error>
where
    T: FromStr,
    T::Err: Display,
    D: Deserializer<'de>,
{
    let s = <String>::deserialize(deserializer)?;
    T::from_str(&s).map_err(Error::custom)
}
