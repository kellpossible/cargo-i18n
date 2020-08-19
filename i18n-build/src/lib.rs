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

pub mod error;
pub mod gettext_impl;
pub mod util;
pub mod watch;

use anyhow::Result;
use i18n_config::Crate;

#[cfg(feature = "localize")]
use lazy_static::lazy_static;

#[cfg(feature = "localize")]
use i18n_embed::{language_loader, DefaultLocalizer, I18nEmbed};

#[cfg(feature = "localize")]
use rust_embed::RustEmbed;

#[cfg(feature = "localize")]
#[derive(RustEmbed, I18nEmbed)]
#[folder = "i18n/mo"]
struct Translations;

#[cfg(feature = "localize")]
language_loader!(I18nBuildLanguageLoader);

#[cfg(feature = "localize")]
lazy_static! {
    static ref LANGUAGE_LOADER: I18nBuildLanguageLoader = I18nBuildLanguageLoader::new();
}

#[cfg(feature = "localize")]
static TRANSLATIONS: Translations = Translations {};

/// Run the i18n build process for the provided crate, which must
/// contain an i18n config.
pub fn run(crt: Crate) -> Result<()> {
    let mut crates: Vec<Crate> = Vec::new();

    let mut parent = crt.find_parent();

    crates.push(crt);

    while parent.is_some() {
        crates.push(parent.unwrap());
        parent = crates.last().unwrap().find_parent();
    }

    crates.reverse();

    let mut crates_iter = crates.iter_mut();

    let mut parent = crates_iter
        .next()
        .expect("expected there to be at least one crate");

    for child in crates_iter {
        child.parent = Some(parent);
        parent = child;
    }

    let last_child_crt = parent;

    let i18n_config = last_child_crt.config_or_err()?;
    if i18n_config.gettext.is_some() {
        gettext_impl::run(last_child_crt)?;
    }

    Ok(())
}

/// Obtain a [Localizer](i18n_embed::Localizer) for localizing this library.
///
/// ⚠️ *This API requires the following crate features to be activated: `localize`.*
#[cfg(feature = "localize")]
pub fn localizer() -> DefaultLocalizer<'static> {
    DefaultLocalizer::new(&*LANGUAGE_LOADER, &TRANSLATIONS)
}
