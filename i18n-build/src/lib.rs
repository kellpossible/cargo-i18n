//! This library is designed for use within the
//! [cargo-i18n](https://crates.io/crates/cargo_i18n) tool for
//! localizing crates. It has been exposed and published as a library
//! to allow its direct use within project build scripts if required.
//!
//! `xtr` (installed with `cargo install xtr`), and GNU Gettext CLI
//! tools `msginit`, `msgfmt`, `msgmerge` and `msgcat` to be present
//! in your system path.
//!
//! # Optional Features
//!
//! The `i18n-build` crate has the following optional Cargo features:
//!
//! + `localize`
//!   + Enables the runtime localization of this library using
//!     [localize()](#localize()) function via the
//!     [i18n-embed](https://crates.io/crates/i18n-embed) crate

mod error;
pub mod gettext_impl;
mod util;
pub mod watch;

use anyhow::Result;

#[cfg(feature = "localize")]
use i18n_embed::{DefaultLocalizer, I18nEmbed, LanguageLoader, Localizer};

#[cfg(feature = "localize")]
use rust_embed::RustEmbed;

use i18n_config;

#[derive(RustEmbed, I18nEmbed)]
#[folder = "i18n/mo"]
struct Translations;

#[derive(LanguageLoader)]
struct I18nBuildLanguageLoader;

const LANGUAGE_LOADER: I18nBuildLanguageLoader = I18nBuildLanguageLoader {};
const TRANSLATIONS: Translations = Translations {};

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

/// Localize this library, and select the language using the provided
/// [LanguageRequester](LanguageRequester).
#[cfg(feature = "localize")]
pub fn localizer() -> Box<dyn Localizer<'static>> {
    Box::from(DefaultLocalizer::new(&LANGUAGE_LOADER, &TRANSLATIONS))
}
