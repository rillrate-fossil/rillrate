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

pub type AddrSender = watch::Sender<Option<SocketAddr>>;

pub type AddrReceiver = watch::Receiver<Option<SocketAddr>>;

pub type AddrChannel = (Mutex<Option<AddrSender>>, AddrReceiver);

fn add_channel() -> AddrChannel {
    let (tx, rx) = watch::channel(None);
    (Mutex::new(Some(tx)), rx)
}

pub(crate) static EXTERN_ADDR: Lazy<AddrChannel> = Lazy::new(add_channel);

pub(crate) static INTERN_ADDR: Lazy<AddrChannel> = Lazy::new(add_channel);
