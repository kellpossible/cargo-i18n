//! `xtr`, and GNU Gettext CLI tools
//! `msginit`, `msgfmt`, `msgmerge` and `msgcat` to be present in your
//! system path.

pub mod config;
mod error;
pub mod gettext;
mod util;
pub mod watch;

use anyhow::Result;

pub fn run(config: &config::I18nConfig) -> Result<()> {
    match config.gettext {
        Some(_) => {
            gettext::run(config)?;
        }
        None => {}
    }

    Ok(())
}
