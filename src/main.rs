mod config;
mod watch;

use config::Config;
use notify::{Error, Result};
use watch::watch;

fn main() -> Result<()> {
    let mut config = Config::new().map_err(|err| Error::generic(&format!("{err}")))?;

    watch(&mut config)
}
