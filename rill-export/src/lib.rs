//! RillExport crate.

#![warn(missing_docs)]

mod actors;
mod config;

pub use actors::embedded_node::EmbeddedNode;
use once_cell::sync::Lazy;
use std::net::SocketAddr;
use tokio::sync::{watch, Mutex};

metacrate::meta!();

// TODO: Refactor that below

/// SocketAddr sender
pub type AddrSender = watch::Sender<Option<SocketAddr>>;

/// SocketAddr receiver
pub type AddrReceiver = watch::Receiver<Option<SocketAddr>>;

/// SocketAddr watch channel pair
pub type AddrChannel = (Mutex<Option<AddrSender>>, AddrReceiver);

fn add_channel() -> AddrChannel {
    let (tx, rx) = watch::channel(None);
    (Mutex::new(Some(tx)), rx)
}

/// External address
pub static EXTERN_ADDR: Lazy<AddrChannel> = Lazy::new(add_channel);

/// Internal address
pub static INTERN_ADDR: Lazy<AddrChannel> = Lazy::new(add_channel);
