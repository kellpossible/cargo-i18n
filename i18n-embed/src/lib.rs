//! Traits and macros to conveniently embed the output of
//! [cargo-i18n](https://crates.io/crates/cargo_i18n) into your
//! application binary in order to localize it at runtime.
//!
//! The core trait for this library is [I18nEmbed](I18nEmbed), which
//! also has a derive macro to allow it to be easily implemented on a
//! struct in your project.
//!
//! This library makes use of
//! [rust-embed](https://crates.io/crates/rust-embed) to perform the
//! actual embedding of the language files, unfortunately using this
//! currently requires you to manually add it as a dependency to your
//! project and implement its trait on your struct in addition to
//! [I18nEmbed](I18nEmbed). At some point in the future this library
//! may incorperate the embedding process into the `I18nEmbed` trait
//! and remove this dependency. [RustEmbed](RustEmbed) currently will
//! not compile if the target `folder` path is invalid, so it is
//! recommended to either run `cargo i18n` before building your
//! project, or committing the compiled resources to ensure that the
//! project can build without requiring `cargo i18n`.
//!
//! # Optional Features
//!
//! The `i18n-embed` crate has the following optional Cargo features:
//!
//! + `desktop-requester`
//!   + Enables a convenience implementation of
//!     [LanguageRequester](LanguageRequester) trait called
//!     [DesktopLanguageRequester](DesktopLanguageRequester) for the
//!     desktop platform (windows, mac, linux), which makes use of
//!     the [locale_config](https://crates.io/crates/locale_config)
//!     crate for resolving the current system locale.
//! + `web-sys-requester`
//!   + Enables a convenience implementation of
//!     [LanguageRequester](LanguageRequester) trait called
//!     [WebLanguageRequester](WebLanguageRequester) which makes use
//!     of the [web-sys](https://crates.io/crates/web-sys) crate for
//!     resolving the language being requested by the user's web
//!     browser in a WASM context.
//!
//! # Examples
//!
//! The following is an example for how to derive the required traits
//! on structs, and localize your binary using this library:
//!
//! ```
//! use i18n_embed::{I18nEmbed, LanguageLoader, DesktopLanguageRequester};
//! use rust_embed::RustEmbed;
//!
//! #[derive(RustEmbed, I18nEmbed)]
//! #[folder = "i18n/mo"] // path to the compiled localization resources
//! struct Translations;
//!
//! #[derive(LanguageLoader)]
//! struct MyLanguageLoader;
//!
//! fn main() {
//!     let language_loader = MyLanguageLoader {};
//!
//!     // Use the language requester for the desktop platform (linux, windows, mac).
//!     // There is also a requester available for the web-sys WASM platform called
//!     // WebLanguageRequester, or you can implement your own.
//!     let language_requester = DesktopLanguageRequester::new();
//!     Translations::select_auto(&language_requester, &language_loader);
//! }
//! ```
//!
//! If you wish to create a localizable library using `i18n-embed`,
//! you can follow this code pattern:
//!
//! ```
//! use i18n_embed::{LanguageRequester, I18nEmbed, LanguageLoader};
//! use rust_embed::RustEmbed;
//!
//! #[derive(RustEmbed, I18nEmbed)]
//! #[folder = "i18n/mo"] // path to the compiled localization resources
//! struct Translations;
//!
//! #[derive(LanguageLoader)]
//! struct MyLanguageLoader;
//!
//! /// Localize this library, and select the language using the provided
//! /// LanguageRequester.
//! pub fn localize<L: LanguageRequester>(language_requester: L) {
//!     let loader = MyLanguageLoader {};
//!     Translations::select_auto(&language_requester, &loader);
//! }
//! ```
//!
//! People using this library can call `localize()` to perform the
//! localization at runtime, and provide their own
//! [LanguageRequester](LanguageRequester) specific to the platform
//! they that they are targetting.
//!
//! If you want to localize a sub-crate in your project, and want to
//! extract strings from this sub-crate and store/embed them in one
//! location in the parent crate, you can use the following pattern
//! for the library:
//!
//! ```
//! use i18n_embed::{LanguageRequester, I18nEmbed, LanguageLoader};
//!
//! #[derive(LanguageLoader)]
//! struct MyLanguageLoader;
//!
//! /// Localize this library, and select the language using the 
//! /// provided I18nEmbed and LanguageRequester.
//! pub fn localize<E: I18nEmbed, L: LanguageRequester>(language_requester: L) {
//!     let loader = MyLanguageLoader {};
//!     E::select_auto(&language_requester, &loader);
//! }
//! ```
//!
//! For the above example, you can enable the following options in the
//! sub-crate's `i18n.toml` to ensure that the localization resources
//! are extracted and merged with the parent crate's `pot` file:
//!
//! ```toml
//! # ...
//!
//! [gettext]
//!
//! # ...
//!
//! # (Optional) If this crate is being localized as a subcrate, store the final
//! # localization artifacts (the module pot and mo files) with the parent crate's
//! # output. Currently crates which contain subcrates with duplicate names are not
//! # supported.
//! extract_to_parent = true
//!
//! # (Optional) If a subcrate has extract_to_parent set to true, then merge the
//! # output pot file of that subcrate into this crate's pot file.
//! collate_extracted_subcrates = true
//! ```

#[cfg(doctest)]
#[macro_use]
extern crate doc_comment;

#[cfg(doctest)]
doctest!("../README.md");

#[allow(unused_imports)]
#[macro_use]
extern crate i18n_embed_impl;
pub use i18n_embed_impl::*;

use std::borrow::Cow;

use fluent_langneg::{negotiate_languages, NegotiationStrategy};
use log::{debug, error, info};
use rust_embed::RustEmbed;
use thiserror::Error;

pub use unic_langid;
pub use tr;
pub use gettext;

#[derive(Error, Debug)]
pub enum I18nEmbedError {
    #[error("Error parsing a language identifier string \"{0}\" because {1}")]
    ErrorParsingLocale(String, #[source] unic_langid::LanguageIdentifierError),
    #[error("The requested language \"{0}\" is not available")]
    LanguageNotAvailable(String)
}

pub trait Localizer<'a> {
    fn language_loader(&self) -> &'a dyn LanguageLoader;
    fn i18n_embed(&self) -> &'a dyn DynamicI18nEmbed;

    fn available_languages(&self) -> Result<Vec<unic_langid::LanguageIdentifier>, I18nEmbedError> {
        self.i18n_embed().available_languages(self.language_loader())
    }

    /// Automatically the language currently requested by the system
    /// by the the [LanguageRequester](LanguageRequester)), and load
    /// it using the provided [LanguageLoader](LanguageLoader).
    fn select(&self, requested_languages: &Vec<unic_langid::LanguageIdentifier>) -> Result<(), I18nEmbedError> {
        select(self.language_loader(), self.i18n_embed(), requested_languages)
    }
}

/// A trait used by [I18nEmbed](I18nEmbed) to ascertain which
/// languages are being requested.
pub trait LanguageRequester<'a> {
    fn add_listener(&mut self, localizer: &'a dyn Localizer<'a>);
    fn poll(&self) -> Result<(), I18nEmbedError>;
    // fn override_request(&self, language_id: unic_langid::LanguageIdentifier);
    // fn reset_override_request(&self);
    // /// Attach a listener to the system this requester is mediating
    // /// for, to listen for changes, and trigger a language update if
    // /// required.
    // fn attach_system_listener(&self) -> Result<(), Box<dyn std::error::Error>>;
    // /// Detach the listener which was attached using
    // /// [attach_change_listener()](#attach_change_lister()).
    // fn detatch_system_listener(&self) -> Result<(), Box<dyn std::error::Error>>;
    /// The currently requested languages.
    fn requested_languages(&self) -> Vec<unic_langid::LanguageIdentifier>;
}

/// Automatically the language currently requested by the system
/// by the the [LanguageRequester](LanguageRequester)), and load
/// it using the provided [LanguageLoader](LanguageLoader).
pub fn select(language_loader: &dyn LanguageLoader, i18n_embed: &dyn DynamicI18nEmbed, requested_languages: &Vec<unic_langid::LanguageIdentifier>) -> Result<(), I18nEmbedError> {
    info!(
        "Selecting translations for domain \"{0}\"",
        language_loader.domain()
    );

    let available_languages: Vec<unic_langid::LanguageIdentifier> = i18n_embed.available_languages(language_loader)?;
    let default_language: unic_langid::LanguageIdentifier = language_loader.src_locale();

    let supported_languages = negotiate_languages(
        &requested_languages,
        &available_languages,
        Some(&default_language),
        NegotiationStrategy::Filtering,
    );

    info!("Requested Languages: {:?}", requested_languages);
    info!("Available Languages: {:?}", available_languages);
    info!("Supported Languages: {:?}", supported_languages);

    match supported_languages.get(0) {
        Some(language_id) => {
            select_single(language_loader, i18n_embed, language_id)?;
        }
        None => {
            // do nothing
        }
    }

    Ok(())
}

pub fn select_single(language_loader: &dyn LanguageLoader, i18n_embed: &dyn DynamicI18nEmbed, language_id: &unic_langid::LanguageIdentifier) -> Result<(), I18nEmbedError> {
    if language_id != &language_loader.src_locale() {
        i18n_embed.load_language_file(language_id, language_loader)?;
    }

    info!("Selected languge \"{0}\"", language_id.to_string());
    Ok(())
}

/// A trait used by [I18nEmbed](I18nEmbed) to load a language file for
/// a specific rust module using a specific localization system. The
/// trait is designed such that the loader could be swapped during
/// runtime, or contain state if required.
pub trait LanguageLoader {
    /// The locale used in the source code for the module this loader
    /// is responsible for.
    fn src_locale(&self) -> unic_langid::LanguageIdentifier;
    /// The domain for the translation.
    fn domain(&self) -> &'static str;
    /// Load the language file corresponding to the module that this
    /// loader requested in `module_path()`.
    fn load_language_file(&self, file: Cow<[u8]>);
    /// Calculate the language file name to use for the given
    /// [LanguageLoader](LanguageLoader).
    fn language_file_name(&self) -> String;
}

pub trait DynamicI18nEmbed {
    fn available_languages<'a>(&self, language_loader: &'a dyn LanguageLoader) -> Result<Vec<unic_langid::LanguageIdentifier>, I18nEmbedError>;
    fn load_language_file<'a>(&self, language_id: &unic_langid::LanguageIdentifier, language_loader: &'a dyn LanguageLoader) -> Result<(), I18nEmbedError>;
}

/// A trait to handle the embedding of software translations within
/// the current binary, and the retrieval/loading of those
/// translations at runtime.
pub trait I18nEmbed: RustEmbed {
    /// Calculate the embedded languages available to be selected for
    /// the module requested by the provided [LanguageLoader](LanguageLoader).
    fn available_languages<'a>(language_loader: &'a dyn LanguageLoader) -> Result<Vec<unic_langid::LanguageIdentifier>, I18nEmbedError> {
        use std::path::{Component, Path};

        let mut language_strings: Vec<String> = Self::iter()
            .map(|filename_cow| filename_cow.to_string())
            .filter_map(|filename| {
                let path: &Path = Path::new(&filename);

                let components: Vec<Component> = path.components().collect();

                let locale: Option<String> = match components.get(0) {
                    Some(language_component) => match language_component {
                        Component::Normal(s) => {
                            Some(s.to_str().expect("path should be valid utf-8").to_string())
                        }
                        _ => None,
                    },
                    _ => None,
                };

                let language_file_name: Option<String> = components
                    .get(1)
                    .map(|component| match component {
                        Component::Normal(s) => {
                            Some(s.to_str().expect("path should be valid utf-8").to_string())
                        }
                        _ => None,
                    })
                    .flatten();

                match language_file_name {
                    Some(language_file_name) => {
                        debug!("Found language file: \"{0}\"", &filename);
                        if language_file_name == language_loader.language_file_name() {
                            locale
                        } else {
                            None
                        }
                    }
                    None => None,
                }
            })
            .collect();

        language_strings.insert(0, language_loader.src_locale().to_string());

        language_strings
            .into_iter()
            .map(|language: String| language.parse().map_err(|err| I18nEmbedError::ErrorParsingLocale(language, err)))
            .collect()
    }

    fn load_language_file<'a>(language_id: &unic_langid::LanguageIdentifier, language_loader: &'a dyn LanguageLoader) -> Result<(), I18nEmbedError> {
        let language_id_string = language_id.to_string();

        let file_path = format!("{}/{}", language_id_string, language_loader.language_file_name());
        let f = Self::get(file_path.as_ref()).ok_or(I18nEmbedError::LanguageNotAvailable(language_id_string))?;
        info!("Loading language \"{0}\" from file \"{1}\"", language_id.to_string(), file_path);
        language_loader.load_language_file(f);
        Ok(())
    }
}

/// A [LanguageRequester](LanguageRequester) for the desktop platform,
/// supporting windows, linux and mac. It uses
/// [locale_config](locale_config) to select the language based on the
/// system selected language.
#[cfg(feature = "desktop-requester")]
pub struct DesktopLanguageRequester<'a> {
    listeners: Vec<&'a dyn Localizer<'a>>
}

#[cfg(feature = "desktop-requester")]
impl <'a> LanguageRequester<'a> for DesktopLanguageRequester<'a> {
    fn requested_languages(&self) -> Vec<unic_langid::LanguageIdentifier> {
        Self::requested_languages()
    }

    fn add_listener(&mut self, localizer: &'a dyn Localizer<'a>) {
        self.listeners.push(localizer);
    }

    fn poll(&self) -> Result<(), I18nEmbedError> { 
        let requested_languages = self.requested_languages();

        for listener in &self.listeners {
            listener.select(&requested_languages)?;
        }

        Ok(())
    }
}

#[cfg(feature = "desktop-requester")]
impl <'a> DesktopLanguageRequester<'a> {
    pub fn new() -> DesktopLanguageRequester<'a> {
        DesktopLanguageRequester {
            listeners: Vec::new()
        }
    }

    pub fn requested_languages() -> Vec<unic_langid::LanguageIdentifier> {
        use locale_config::{LanguageRange, Locale};

        let current_locale = Locale::current();

        let ids: Vec<unic_langid::LanguageIdentifier> = current_locale
            .tags_for("messages")
            .filter_map(|tag: LanguageRange| match tag.to_string().parse() {
                Ok(tag) => Some(tag),
                Err(err) => {
                    error!("Unable to parse your locale: {:?}", err);
                    None
                }
            })
            .collect();

        info!("Current Locale: {:?}", ids);

        ids
    }
}

/// A [LanguageRequester](LanguageRequester) for the `web-sys` web platform.
#[cfg(feature = "web-sys-requester")]
pub struct WebLanguageRequester;

#[cfg(feature = "web-sys-requester")]
impl WebLanguageRequester {
    pub fn new() -> WebLanguageRequester {
        WebLanguageRequester {}
    }
}

#[cfg(feature = "web-sys-requester")]
impl LanguageRequester for WebLanguageRequester {
    fn requested_languages(&self) -> Vec<unic_langid::LanguageIdentifier> {
        use fluent_langneg::convert_vec_str_to_langids_lossy;
        let window = web_sys::window().expect("no global `window` exists");
        let navigator = window.navigator();
        let languages = navigator.languages();

        let requested_languages =
            convert_vec_str_to_langids_lossy(languages.iter().map(|js_value| {
                js_value
                    .as_string()
                    .expect("language value should be a string.")
            }));

        requested_languages
    }
}

/// Get the translation domain from the module path (first module in
/// the module path).
pub fn domain_from_module(module_path: &str) -> &str {
    module_path.split("::").next().unwrap_or(module_path)
}
