use super::Session;
use derive_more::From;
use meio::prelude::Address;

#[derive(Debug, Clone, From)]
pub struct SessionLink {
    address: Address<Session>,
}
