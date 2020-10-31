use std::thread;

mod connector;
pub mod protocol;
mod provider;
mod worker;

pub fn install() {
    thread::spawn(worker::entrypoint);
}
