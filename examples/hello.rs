use anyhow::Error;
use std::sync::{
    atomic::{AtomicBool, Ordering},
    Arc,
};
use std::thread;
use std::time::{Duration, Instant};

fn main() -> Result<(), Error> {
    env_logger::try_init()?;
    rill::install("example-hello")?;
    // TODO: Fix it
    //rill::awake(&module_1::RILL);
    //rill::awake(&module_2::RILL);
    let running = Arc::new(AtomicBool::new(true));
    let r = running.clone();
    ctrlc::set_handler(move || {
        r.store(false, Ordering::SeqCst);
    })
    .expect("Error setting Ctrl-C handler");
    while running.load(Ordering::SeqCst) {
        module_1::work();
        module_2::work();
        //log::trace!("Cool!");
        //log::warn!("Nice!");
        thread::sleep(Duration::from_millis(10));
        //log::trace!("PING: {:?}", Instant::now());
    }
    rill::terminate()?;
    Ok(())
}

mod module_1 {
    rill::provider!();

    pub fn work() {
        rill::log!("work module_1 called".into());
    }
}

mod module_2 {
    rill::provider!(public);

    pub fn work() {
        rill::log!("work module_2 called".into());
    }
}
