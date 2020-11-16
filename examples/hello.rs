use anyhow::Error;
use std::thread;
use std::time::{Duration, Instant};

fn main() -> Result<(), Error> {
    env_logger::try_init()?;
    rill::install("example-hello")?;
    rill::awake(&module_1::RILL);
    rill::awake(&module_2::RILL);
    loop {
        module_1::work();
        module_2::work();
        log::trace!("Cool!");
        log::warn!("Nice!");
        thread::sleep(Duration::from_millis(10));
        log::trace!("PING: {:?}", Instant::now());
    }
}

mod module_1 {
    rill::provider!();

    pub fn work() {
        rill::log!("work module_1 called".into());
    }
}

mod module_2 {
    rill::provider!();

    pub fn work() {
        rill::log!("work module_2 called".into());
    }
}
