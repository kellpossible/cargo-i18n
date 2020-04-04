//! `xtr`, and GNU Gettext CLI tools
//! `msginit`, `msgfmt`, `msgmerge` and `msgcat` to be present in your
//! system path.

pub mod config;
mod error;
pub mod gettext_impl;
mod util;
pub mod watch;

use anyhow::Result;

#[cfg(feature = "localize")]
use i18n_embed::I18nEmbed;

#[cfg(feature = "localize")]
use gettext::Catalog;

#[cfg(feature = "localize")]
use tr::set_translator;

pub fn run(crt: &config::Crate) -> Result<()> {
    let i18n_config = crt.config_or_err()?;
    match i18n_config.gettext {
        Some(_) => {
            gettext_impl::run(crt)?;
        }
        None => {}
    }

    Ok(())
}

struct LanguageLoader;

impl LanguageLoader {
    fn new() -> LanguageLoader {
        LanguageLoader {}
    }
}

impl i18n_embed::LanguageLoader for LanguageLoader {
    fn load_language_file<R: std::io::Read>(&self, reader: R) {
        let catalog = Catalog::parse(reader).expect("could not parse the catalog");
        set_translator!(catalog);
    }

    fn module_path() -> &'static str {
        module_path!()
    }
}

pub fn localize<E: I18nEmbed>() {
    let loader = LanguageLoader::new();
    E::select(&loader);
}
