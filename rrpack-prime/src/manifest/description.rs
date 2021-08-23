use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct Weight {
    pub group: u16,
    pub item: u16,
}

impl From<u16> for Weight {
    fn from(value: u16) -> Self {
        let group = value / 100;
        let item = value % 100;
        Self { group, item }
    }
}
