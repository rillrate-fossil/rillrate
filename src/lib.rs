use std::thread;

mod worker;

pub fn install() {
    thread::spawn(worker::entrypoint);
}
