use crate::storage::typed_storage::Storable;
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
pub struct AuthToken(pub String);

impl Storable for AuthToken {
    fn key() -> &'static str {
        "rillrate.storage.auth-token"
    }
}
