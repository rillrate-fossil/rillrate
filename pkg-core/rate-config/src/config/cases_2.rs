use ordered_float::OrderedFloat;
use rill_protocol::io::provider::Path;
use rrpack_basis::manifest::layouts::components as basis;
use serde::{de::Error, Deserialize, Deserializer};
use std::fmt::Display;
use std::str::FromStr;

type BoxedElement = Box<Element>;

#[derive(Debug, Clone, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "kebab-case")]
pub enum Element {
    //Align(Align),
    Label(Label),
    Flow(Flow),
}

impl From<BoxedElement> for basis::Element {
    fn from(value: BoxedElement) -> Self {
        match *value {
            //Element::Container(value) => Self::Container(Box::new((*value).into())),
            //Element::Align(value) => Self::Align(value.into()),
            Element::Label(value) => Self::Label(value.into()),
            Element::Flow(value) => Self::Flow(value.into()),
            //Element::Element(value) => value.into(),
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
            child: value.child.into(),
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
