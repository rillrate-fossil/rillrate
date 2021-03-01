use super::RillClient;
use anyhow::Error;
use async_trait::async_trait;
use derive_more::From;
use meio::{Action, Address};

#[derive(Debug, From)]
pub struct ClientLink {
    address: Address<RillClient>,
}
