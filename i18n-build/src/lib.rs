//! `xtr`, and GNU Gettext CLI tools
//! `msginit`, `msgfmt`, `msgmerge` and `msgcat` to be present in your
//! system path.

mod error;
pub mod gettext_impl;
mod util;
pub mod watch;

use anyhow::Result;

#[cfg(feature = "localize")]
use i18n_embed::{DesktopLanguageRequester, I18nEmbed};

#[cfg(feature = "localize")]
use gettext::Catalog;

#[cfg(feature = "localize")]
use tr::set_translator;

use i18n_config;

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

/// Localize this library using the provided [I18nEmbed](I18nEmbed)
/// implementation.
///
/// TODO: consider moving the translations and embedding them directly
/// in the library, rather than as a sub-crate.
#[cfg(feature = "localize")]
pub fn localize<E: I18nEmbed>() {
    let loader = LanguageLoader::new();
    let requester = DesktopLanguageRequester::new();
    E::select(&requester, &loader);
}
