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
#[serde(rename_all = "kebab-case")]
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

impl From<Element> for basis::Element {
    fn from(value: Element) -> Self {
        match value {
            Element::Empty => Self::Empty,

            // Containers
            Element::Align(value) => Self::Align(value.into()),
            Element::Center(value) => Self::Center(value.into()),
            Element::Container(value) => Self::Container(value.into()),
            Element::Expanded(value) => Self::Expanded(value.into()),
            Element::Spacer(value) => Self::Spacer(value.into()),
            Element::Row(value) => Self::Row(value.into()),
            Element::Column(value) => Self::Column(value.into()),

            // Components
            Element::Text(value) => Self::Text(value.into()),
            Element::Flow(value) => Self::Flow(value.into()),
        }
    }
}

#[derive(Debug, Clone, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "kebab-case")]
pub struct Align {
    pub alignment: Alignment,
    pub child: BoxedElement,
}

impl From<Align> for basis::Align {
    fn from(value: Align) -> Self {
        Self {
            alignment: value.alignment.into(),
            child: value.child.into(),
        }
    }
}

#[derive(Debug, Clone, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "kebab-case")]
pub struct Center {
    #[serde(rename = "$value")]
    pub child: BoxedElement,
}

impl From<Center> for basis::Center {
    fn from(value: Center) -> Self {
        Self {
            child: value.child.into(),
        }
    }
}

#[derive(Debug, Clone, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "kebab-case")]
pub struct Container {
    pub child: BoxedElement,
}

impl From<Container> for basis::Container {
    fn from(value: Container) -> Self {
        Self {
            child: value.child.into(),
        }
    }
}

#[derive(Debug, Clone, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "kebab-case")]
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
#[serde(rename_all = "kebab-case")]
pub struct Spacer {
    pub flex: Option<OrderedFloat<f64>>,
    pub maintenance: Option<bool>,
}

impl From<Spacer> for basis::Spacer {
    fn from(value: Spacer) -> Self {
        // TODO: How to improve default?
        Self {
            flex: value.flex.unwrap_or_else(|| OrderedFloat(1.0)),
            maintenance: value.maintenance.unwrap_or_default(),
        }
    }
}

#[derive(Debug, Clone, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "kebab-case")]
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
#[serde(rename_all = "kebab-case")]
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
#[serde(rename_all = "kebab-case")]
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
#[serde(rename_all = "kebab-case")]
pub struct Text {
    #[serde(rename = "$value")]
    pub text: String,
    pub align: Option<TextAlign>,
}

impl From<Text> for basis::Text {
    fn from(value: Text) -> Self {
        Self {
            text: value.text,
            align: value.align.unwrap_or_default().into(),
        }
    }
}

#[derive(Debug, Clone, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "kebab-case")]
pub enum TextAlign {
    Left = 0,
    Right = 1,
    Center = 2,
    Justify = 3,
    Start = 4,
    End = 5,
}

// TODO: Move defaults to the `basis`?
impl Default for TextAlign {
    fn default() -> Self {
        Self::Left
    }
}

impl From<TextAlign> for basis::TextAlign {
    fn from(value: TextAlign) -> Self {
        match value {
            TextAlign::Left => Self::Left,
            TextAlign::Right => Self::Right,
            TextAlign::Center => Self::Center,
            TextAlign::Justify => Self::Justify,
            TextAlign::Start => Self::Start,
            TextAlign::End => Self::End,
        }
    }
}

#[derive(Debug, Clone, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "kebab-case")]
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
