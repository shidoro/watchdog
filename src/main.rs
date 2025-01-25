mod config;
mod root;
mod watch;

use config::Config;
use notify::{Error, Result as NotifyResult};
use watch::watch;

fn main() -> NotifyResult<()> {
    let mut config = Config::new().map_err(|err| Error::generic(&format!("{err}")))?;

    watch(&mut config)
}
