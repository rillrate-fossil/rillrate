use rillrate::RillRate;
use std::error::Error;

fn main() -> Result<(), Box<dyn Error>> {
    let _rillrate = RillRate::from_env("my-app")?;
    Ok(())
}
