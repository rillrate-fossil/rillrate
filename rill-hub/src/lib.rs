//! RillExport crate.

#![warn(missing_docs)]

mod actors;
pub mod config;

pub use actors::hub::RillHub;
use once_cell::sync::Lazy;
use std::net::SocketAddr;
use tokio::sync::{oneshot, Mutex};

metacrate::meta!();

// TODO: Refactor that below

/// SocketAddr sender
pub type AddrSender = oneshot::Sender<SocketAddr>;

/// SocketAddr receiver
pub type AddrReceiver = oneshot::Receiver<SocketAddr>;

/// SocketAddr oneshot channel pair
pub type AddrChannel = Mutex<(Option<AddrSender>, Option<AddrReceiver>)>;

fn add_channel() -> AddrChannel {
    let (tx, rx) = oneshot::channel();
    Mutex::new((Some(tx), Some(rx)))
}

/// Reference to an address channel
pub type AddrCell = &'static Lazy<AddrChannel>;

/// External address
pub static EXTERN_ADDR: Lazy<AddrChannel> = Lazy::new(add_channel);

/// Internal address
pub static INTERN_ADDR: Lazy<AddrChannel> = Lazy::new(add_channel);
