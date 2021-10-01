use ordered_float::OrderedFloat;
use rill_protocol::io::provider::Path;
use rrpack_basis::manifest::layouts::components as basis;
use serde::{de::Error, Deserialize, Deserializer, Serialize};
use std::fmt::Display;
use std::str::FromStr;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct Layout {
    #[serde(deserialize_with = "from_str")]
    pub name: Path,
    #[serde(rename = "$value")]
    pub container: Container,
}

impl From<Layout> for basis::Layout {
    fn from(value: Layout) -> Self {
        Self {
            name: value.name,
            container: value.container.into(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum Container {
    Empty,
    Align(Align),
    Expanded(Expanded),
    Spacer(Spacer),
    Row(Row),
    Column(Column),
}

impl From<Container> for basis::Container {
    fn from(value: Container) -> Self {
        match value {
            Container::Empty => Self::Empty,
            Container::Align(value) => Self::Align(value.into()),
            Container::Expanded(value) => Self::Expanded(value.into()),
            Container::Spacer(value) => Self::Spacer(value.into()),
            Container::Row(value) => Self::Row(value.into()),
            Container::Column(value) => Self::Column(value.into()),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub struct Align {
    pub alignment: Alignment,
    pub child: Element,
}

impl From<Align> for basis::Align {
    fn from(value: Align) -> Self {
        Self {
            alignment: value.alignment.into(),
            child: value.child.into(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub struct Expanded {
    pub child: Element,
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

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub struct Spacer {
    pub flex: OrderedFloat<f64>,
}

impl From<Spacer> for basis::Spacer {
    fn from(value: Spacer) -> Self {
        Self { flex: value.flex }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
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

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
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

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
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

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum Element {
    Container(Box<Container>),
    Label(Label),
    Flow(Flow),
}

impl From<Element> for basis::Element {
    fn from(value: Element) -> Self {
        match value {
            Element::Container(value) => Self::Container(Box::new((*value).into())),
            Element::Label(value) => Self::Label(value.into()),
            Element::Flow(value) => Self::Flow(value.into()),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
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

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
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
