use std::thread;

pub mod protocol;
mod worker;

pub fn install() {
    thread::spawn(worker::entrypoint);
}
