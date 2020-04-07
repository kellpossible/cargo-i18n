//! `xtr`, and GNU Gettext CLI tools
//! `msginit`, `msgfmt`, `msgmerge` and `msgcat` to be present in your
//! system path.

mod error;
pub mod gettext_impl;
mod util;
pub mod watch;

use anyhow::Result;

#[cfg(feature = "localize")]
use i18n_embed::{LanguageRequester, I18nEmbed};

#[cfg(feature = "localize")]
use gettext::Catalog;

#[cfg(feature = "localize")]
use tr::set_translator;

#[cfg(feature = "localize")]
use rust_embed::RustEmbed;

use i18n_config;

#[derive(RustEmbed, I18nEmbed)]
#[folder = "i18n/mo"]
#[cfg(feature = "localize")]
struct Translations;

/// Run the i18n build process for the provided crate, which must
/// contain an i18n config.
pub fn run(crt: &i18n_config::Crate) -> Result<()> {
    let i18n_config = crt.config_or_err()?;
    match i18n_config.gettext {
        Some(_) => {
            gettext_impl::run(crt)?;
        }
        None => {}
    }

    Ok(())
}

#[cfg(feature = "localize")]
struct LanguageLoader;

#[cfg(feature = "localize")]
impl LanguageLoader {
    fn new() -> LanguageLoader {
        LanguageLoader {}
    }
}

// TODO: change when we have more embed macros
#[cfg(feature = "localize")]
impl i18n_embed::LanguageLoader for LanguageLoader {
    fn load_language_file<R: std::io::Read>(&self, reader: R) {
        let catalog = Catalog::parse(reader).expect("could not parse the catalog");
        set_translator!(catalog);
    }

    fn module_path() -> &'static str {
        module_path!()
    }
}

/// Localize this library, and select the language using the provided
/// [LanguageRequester](LanguageRequester).
#[cfg(feature = "localize")]
pub fn localize<L: LanguageRequester>(language_requester: L) {
    let loader = LanguageLoader::new();
    Translations::select(&language_requester, &loader);
}
