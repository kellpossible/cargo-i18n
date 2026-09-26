#![doc = include_str!("../README.md")]
#![cfg_attr(docsrs, feature(doc_cfg))]

pub mod error;
pub mod gettext_impl;
pub mod util;
pub mod watch;

use anyhow::Result;
use i18n_config::Crate;

/// Run the i18n build process for the provided crate, which must
/// contain an i18n config.
pub fn run(crt: Crate<'_>) -> Result<()> {
    let mut crates: Vec<Crate<'_>> = Vec::new();

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

#[cfg(feature = "localize")]
#[cfg_attr(docsrs, doc(cfg(feature = "localize")))]
mod localize_feature {
    use i18n_embed::{
        DefaultLocalizer,
        gettext::{GettextLanguageLoader, gettext_language_loader},
    };
    use std::sync::OnceLock;

    use rust_embed::RustEmbed;

    #[derive(RustEmbed)]
    #[folder = "i18n/mo"]
    struct Translations;

    static TRANSLATIONS: Translations = Translations {};

    fn language_loader() -> &'static GettextLanguageLoader {
        static LANGUAGE_LOADER: OnceLock<GettextLanguageLoader> = OnceLock::new();

        LANGUAGE_LOADER.get_or_init(|| gettext_language_loader!())
    }

    /// Obtain a [Localizer](i18n_embed::Localizer) for localizing this library.
    pub fn localizer() -> DefaultLocalizer<'static> {
        DefaultLocalizer::new(language_loader(), &TRANSLATIONS)
    }
}

#[cfg(feature = "localize")]
#[cfg_attr(docsrs, doc(cfg(feature = "localize")))]
pub use localize_feature::localizer;
