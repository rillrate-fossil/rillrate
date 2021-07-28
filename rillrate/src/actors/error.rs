use thiserror::Error;

#[derive(Debug, Error)]
#[error("inner value already taken")]
pub struct AlreadyTaken;

#[derive(Debug, Error)]
#[error("inner value was not set")]
pub struct NotSet;
