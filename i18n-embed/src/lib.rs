#![allow(clippy::needless_doctest_main)]
//! Traits and macros to conveniently embed localization assets into
//! your application binary or library in order to localize it at
//! runtime. Works in unison with
//! [cargo-i18n](https://crates.io/crates/cargo_i18n).
//!
//! This library recommends tha you make use of
//! [rust-embed](https://crates.io/crates/rust-embed) to perform the
//! actual embedding of the language files, unfortunately using this
//! currently requires you to manually add it as a dependency to your
//! project and implement its trait on your struct in addition to
//! [I18nAssets](I18nAssets). `RustEmbed` will not compile if the
//! target `folder` path is invalid, so it is recommended to either
//! run `cargo i18n` before building your project, or committing the
//! localization assets into source control to ensure that the the
//! folder exists and project can build without requiring `cargo
//! i18n`.
//!
//! # Optional Features
//!
//! The `i18n-embed` crate has the following optional Cargo features:
//!
//! + `rust-embed` (Enabled by default)
//!   + Enable an automatic implementation of [I18nAssets] for any
//!     type that also implements `RustEmbed`.
//! + `fluent-system`
//!   + Enable support for the
//!     [fluent](https://www.projectfluent.org/) localization system
//!     via the `fluent::FluentLanguageLoader` in this crate.
//! + `gettext-system`
//!   + Enable support for the
//!     [gettext](https://www.gnu.org/software/gettext/) localization
//!     system using the [tr macro](https://docs.rs/tr/0.1.3/tr/) and
//!     the [gettext crate](https://docs.rs/gettext/0.4.0/gettext/)
//!     via the `gettext::GettextLanguageLoader` in this crate.
//! + `desktop-requester`
//!   + Enables a convenience implementation of
//!     [LanguageRequester](LanguageRequester) trait called
//!     `DesktopLanguageRequester for the desktop platform (windows,
//!     mac, linux), which makes use of the
//!     [locale_config](https://crates.io/crates/locale_config) crate
//!     for resolving the current system locale.
//! + `web-sys-requester`
//!   + Enables a convenience implementation of
//!     [LanguageRequester](LanguageRequester) trait called
//!     `WebLanguageRequester` which makes use of the
//!     [web-sys](https://crates.io/crates/web-sys) crate for
//!     resolving the language being requested by the user's web
//!     browser in a WASM context.
//!
//! # Examples
//!
//! ## Fluent Localization System
//!
//! The following is a simple example for how to localize your binary
//! using this library when it first runs, using the `fluent`
//! localization system, directly instantiating the
//! `FluentLanguageLoader`.
//!
//! First you'll need the following features enabled in your
//! `Cargo.toml`:
//!
//! ```toml
//! [dependencies]
//! i18n-embed = { version = "VERSION", features = ["fluent-system", "desktop-requester"]}
//! rust-embed = "6"
//! ```
//!
//! Set up a minimal `i18n.toml` in your crate root to use with
//! `cargo-i18n` (see [cargo
//! i18n](https://github.com/kellpossible/cargo-i18n#configuration)
//! for more information on the configuration file format):
//!
//! ```toml
//! # (Required) The language identifier of the language used in the
//! # source code for gettext system, and the primary fallback language
//! # (for which all strings must be present) when using the fluent
//! # system.
//! fallback_language = "en-GB"
//!
//! # Use the fluent localization system.
//! [fluent]
//! # (Required) The path to the assets directory.
//! # The paths inside the assets directory should be structured like so:
//! # `assets_dir/{language}/{domain}.ftl`
//! assets_dir = "i18n"
//! ```
//!
//! Next, you want to create your localization resources, per language
//! fluent (`.ftl`) files. `language` needs to conform to the [Unicode
//! Language
//! Identifier](https://unicode.org/reports/tr35/tr35.html#Unicode_language_identifier)
//! standard, and will be parsed via the [unic_langid
//! crate](https://docs.rs/unic-langid/0.9.0/unic_langid/):
//!
//! ```txt
//! my_crate/
//!   Cargo.toml
//!   i18n.toml
//!   src/
//!   i18n/
//!     {language}/
//!       {domain}.ftl
//! ```
//!
//! Then in your Rust code:
//!
//! ```
//! use i18n_embed::{DesktopLanguageRequester, fluent::{
//!     FluentLanguageLoader, fluent_language_loader
//! }};
//! use rust_embed::RustEmbed;
//!
//! #[derive(RustEmbed)]
//! #[folder = "i18n"] // path to the compiled localization resources
//! struct Localizations;
//!
//!
//! fn main() {
//!     let language_loader: FluentLanguageLoader = fluent_language_loader!();
//!
//!     // Use the language requester for the desktop platform (linux, windows, mac).
//!     // There is also a requester available for the web-sys WASM platform called
//!     // WebLanguageRequester, or you can implement your own.
//!     let requested_languages = DesktopLanguageRequester::requested_languages();
//!
//!     let _result = i18n_embed::select(
//!         &language_loader, &Localizations, &requested_languages);
//!
//!     // continue on with your application
//! }
//! ```
//!
//! To access localizations, you can use `FluentLanguageLoader`'s
//! methods directly, or, for added compile-time checks/safety, you
//! can use the [fl!() macro](https://crates.io/crates/i18n-embed-fl).
//!
//! Having an `i18n.toml` configuration file enables you to do the
//! following:
//!
//! + Use the [cargo i18n](https://crates.io/crates/cargo-i18n) tool
//!   to perform validity checks (not yet implemented).
//! + Integrate with a code-base using the `gettext` localization
//!   system.
//! + Use the `fluent::fluent_language_loader!()` macro to pull the
//!   configuration in at compile time to create the
//!   `fluent::FluentLanguageLoader`.
//! + Use the [fl!() macro](https://crates.io/crates/i18n-embed-fl) to
//!   have added compile-time safety when accessing messages.
//!
//! ## Gettext Localization System
//!
//! The following is a simple example for how to localize your binary
//! using this library when it first runs, using the `gettext`
//! localization system. Please note that the `gettext` localization
//! system is technically inferior to `fluent` [in a number of
//! ways](https://github.com/projectfluent/fluent/wiki/Fluent-vs-gettext),
//! however there are always legacy reasons, and the
//! developer/translator ecosystem around `gettext` is mature.
//!
//! The `gettext::GettextLanguageLoader` in this example is
//! instantiated using the `gettext::gettext_language_loader!()`
//! macro, which automatically determines the correct module for the
//! crate, and pulls settings in from the `i18n.toml` configuration
//! file.
//!
//! First you'll need the following features enabled in your
//! `Cargo.toml`:
//!
//! ```toml
//! [dependencies]
//! i18n-embed = { version = "VERSION", features = ["gettext-system", "desktop-requester"]}
//! rust-embed = "6"
//! ```
//!
//! Set up a minimal `i18n.toml` in your crate root to use with
//! `cargo-i18n` (see [cargo
//! i18n](https://github.com/kellpossible/cargo-i18n#configuration)
//! for more information on the configuration file format):
//!
//! ```toml
//! # (Required) The language identifier of the language used in the
//! # source code for gettext system, and the primary fallback language
//! # (for which all strings must be present) when using the fluent
//! # system.
//! fallback_language = "en"
//!
//! # Use the gettext localization system.
//! [gettext]
//! # (Required) The languages that the software will be translated into.
//! target_languages = ["es"]
//!
//! # (Required) Path to the output directory, relative to `i18n.toml` of
//! # the crate being localized.
//! output_dir = "i18n"
//! ```
//!
//! Install and run [cargo i18n](https://crates.io/crates/cargo-i18n)
//! for your crate to generate the language specific `po` and `mo`
//! files, ready to be translated. It is recommended to add the
//! `i18n/pot` folder to your repository gitignore.
//!
//! Then in your Rust code:
//!
//! ```
//! use i18n_embed::{DesktopLanguageRequester, gettext::{
//!     gettext_language_loader
//! }};
//! use rust_embed::RustEmbed;
//!
//! #[derive(RustEmbed)]
//! // path to the compiled localization resources,
//! // as determined by i18n.toml settings
//! #[folder = "i18n/mo"]
//! struct Localizations;
//!
//!
//! fn main() {
//!     // Create the GettextLanguageLoader, pulling in settings from `i18n.toml`
//!     // at compile time using the macro.
//!     let language_loader = gettext_language_loader!();
//!
//!     // Use the language requester for the desktop platform (linux, windows, mac).
//!     // There is also a requester available for the web-sys WASM platform called
//!     // WebLanguageRequester, or you can implement your own.
//!     let requested_languages = DesktopLanguageRequester::requested_languages();
//!
//!     let _result = i18n_embed::select(
//!         &language_loader, &Localizations, &requested_languages);
//!
//!     // continue on with your application
//! }
//! ```
//!
//! ## Automatic Updating Selection
//!
//! Depending on the platform, you can also make use of the
//! [LanguageRequester](LanguageRequester)'s ability to monitor
//! changes to the currently requested language, and automatically
//! update the selected language using a [Localizer](Localizer):
//!
//! ```
//! use std::sync::Arc;
//! use i18n_embed::{
//!     DesktopLanguageRequester, LanguageRequester,
//!     DefaultLocalizer, Localizer, fluent::FluentLanguageLoader     
//! };
//! use rust_embed::RustEmbed; use lazy_static::lazy_static;
//! use unic_langid::LanguageIdentifier;
//!
//! #[derive(RustEmbed)]
//! #[folder = "i18n/ftl"] // path to localization resources
//! struct Localizations;
//!
//! lazy_static! {
//!     static ref LANGUAGE_LOADER: FluentLanguageLoader = {
//!         // Usually you could use the fluent_language_loader!() macro
//!         // to pull values from i18n.toml configuration and current
//!         // module here at compile time, but instantiating the loader
//!         // manually here instead so the example compiles.
//!         let fallback: LanguageIdentifier = "en-US".parse().unwrap();
//!         FluentLanguageLoader::new("test", fallback)
//!     };
//! }
//!
//! fn main() {
//!     let localizer = DefaultLocalizer::new(&*LANGUAGE_LOADER, &Localizations);
//!
//!     let localizer_arc: Arc<dyn Localizer> = Arc::new(localizer);
//!
//!     let mut language_requester = DesktopLanguageRequester::new();
//!     language_requester.add_listener(Arc::downgrade(&localizer_arc));
//!
//!     // Manually check the currently requested system language,
//!     // and update the listeners. NOTE: Support for this across systems
//!     // currently varies, it may not change when the system requested
//!     // language changes during runtime without restarting your application.
//!     // In the future some platforms may also gain support for
//!     // automatic triggering when the requested display language changes.
//!     language_requester.poll().unwrap();
//!
//!     // continue on with your application
//! }
//! ```
//!
//! The above example makes use of the
//! [DefaultLocalizer](DefaultLocalizer), but you can also implement
//! the [Localizer](Localizer) trait yourself for a custom solution.
//! It also makes use of
//! [lazy_static](https://crates.io/crates/lazy_static) to allow the
//! [LanguageLoader](LanguageLoader) implementation to be stored
//! statically, because its constructor is not `const`.
//!
//! ## Localizing Libraries
//!
//! If you wish to create a localizable library using `i18n-embed`,
//! you can follow this code pattern in the library itself:
//!
//! ```
//! use std::sync::Arc;
//! use i18n_embed::{
//!     DefaultLocalizer, Localizer, LanguageLoader,
//!     fluent::{
//!         fluent_language_loader, FluentLanguageLoader     
//! }};
//! use rust_embed::RustEmbed; use lazy_static::lazy_static;
//!
//! #[derive(RustEmbed)]
//! #[folder = "i18n/mo"] // path to the compiled localization resources
//! struct Localizations;
//!
//! lazy_static! {
//!     static ref LANGUAGE_LOADER: FluentLanguageLoader = {
//!         let loader = fluent_language_loader!();
//!
//!         // Load the fallback langauge by default so that users of the
//!         // library don't need to if they don't care about localization.
//!         // This isn't required for the `gettext` localization system.
//!         loader.load_fallback_language(&Localizations)
//!             .expect("Error while loading fallback language");
//!
//!         loader
//!     };
//! }
//!
//! // Get the `Localizer` to be used for localizing this library.
//! # #[allow(unused)]
//! pub fn localizer() -> Arc<dyn Localizer> {
//!     Arc::new(DefaultLocalizer::new(&*LANGUAGE_LOADER, &Localizations))
//! }
//! ```
//!
//! People using this library can call `localize()` to obtain a
//! [Localizer](Localizer), and add this as a listener to their chosen
//! [LanguageRequester](LanguageRequester).
//!
//! ## Localizing Sub-crates
//!
//! If you want to localize a sub-crate in your project, and want to
//! extract strings from this sub-crate and store/embed them in one
//! location in the parent crate, you can use the following pattern
//! for the library:
//!
//! ```
//! use std::sync::Arc;
//! use i18n_embed::{
//!     DefaultLocalizer, Localizer, gettext::{
//!     gettext_language_loader, GettextLanguageLoader     
//! }};
//! use i18n_embed::I18nAssets;
//! use lazy_static::lazy_static;
//!
//! lazy_static! {
//!     static ref LANGUAGE_LOADER: GettextLanguageLoader =
//!         gettext_language_loader!();
//! }
//!
//! /// Get the `Localizer` to be used for localizing this library,
//! /// using the provided embeddes source of language files `embed`.
//! # #[allow(unused)]
//! pub fn localizer(embed: &dyn I18nAssets) -> Arc<dyn Localizer + '_> {
//!     Arc::new(DefaultLocalizer::new(
//!         &*LANGUAGE_LOADER,
//!         embed
//!     ))
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

#![doc(test(
    no_crate_inject,
    attr(deny(warnings, rust_2018_idioms, single_use_lifetimes))
))]
#![forbid(unsafe_code)]
#![warn(
    missing_debug_implementations,
    missing_docs,
    rust_2018_idioms,
    single_use_lifetimes,
    unreachable_pub
)]

mod assets;
mod requester;
mod util;

#[cfg(feature = "fluent-system")]
pub mod fluent;

#[cfg(feature = "gettext-system")]
pub mod gettext;

pub use assets::*;
pub use requester::*;
pub use util::*;

#[cfg(doctest)]
#[macro_use]
extern crate doc_comment;

#[cfg(doctest)]
doctest!("../README.md");

#[cfg(any(feature = "gettext-system", feature = "fluent-system"))]
#[allow(unused_imports)]
#[macro_use]
extern crate i18n_embed_impl;

use std::{
    borrow::Cow,
    fmt::Debug,
    path::{Component, Path},
    string::FromUtf8Error,
};

use fluent_langneg::{negotiate_languages, NegotiationStrategy};
use log::{debug, error};
use thiserror::Error;

pub use unic_langid;

/// An error that occurs in this library.
#[derive(Error, Debug)]
#[allow(missing_docs)]
pub enum I18nEmbedError {
    #[error("Error parsing a language identifier string \"{0}\"")]
    ErrorParsingLocale(String, #[source] unic_langid::LanguageIdentifierError),
    #[error("Error reading language file \"{0}\" as utf8.")]
    ErrorParsingFileUtf8(String, #[source] FromUtf8Error),
    #[error("The slice of requested languages cannot be empty.")]
    RequestedLanguagesEmpty,
    #[error("The language file \"{0}\" for the language \"{1}\" is not available.")]
    LanguageNotAvailable(String, unic_langid::LanguageIdentifier),
    #[error("There are multiple errors: {}", error_vec_to_string(.0))]
    Multiple(Vec<I18nEmbedError>),
    #[cfg(feature = "gettext-system")]
    #[error(transparent)]
    Gettext(#[from] gettext_system::Error),
}

fn error_vec_to_string(errors: &[I18nEmbedError]) -> String {
    let strings: Vec<String> = errors.iter().map(|e| format!("{}", e)).collect();
    strings.join(", ")
}

/// This trait provides dynamic access to an
/// [LanguageLoader](LanguageLoader) and an [I18nAssets](I18nAssets),
/// which are used together to localize a library/crate on demand.
pub trait Localizer {
    /// The [LanguageLoader] used by this localizer.
    fn language_loader(&self) -> &'_ dyn LanguageLoader;

    /// The source of localization assets used by this localizer
    fn i18n_assets(&self) -> &'_ dyn I18nAssets;

    /// The available languages that can be selected by this localizer.
    fn available_languages(&self) -> Result<Vec<unic_langid::LanguageIdentifier>, I18nEmbedError> {
        self.language_loader()
            .available_languages(self.i18n_assets())
    }

    /// Automatically the language currently requested by the system
    /// by the the [LanguageRequester](LanguageRequester)), and load
    /// it using the provided [LanguageLoader](LanguageLoader).
    fn select(
        &self,
        requested_languages: &[unic_langid::LanguageIdentifier],
    ) -> Result<Vec<unic_langid::LanguageIdentifier>, I18nEmbedError> {
        select(
            self.language_loader(),
            self.i18n_assets(),
            requested_languages,
        )
    }
}

/// A simple default implemenation of the [Localizer](Localizer) trait.
pub struct DefaultLocalizer<'a> {
    /// The [LanguageLoader] used by this localizer.
    pub language_loader: &'a dyn LanguageLoader,
    /// The source of assets used by this localizer.
    pub i18n_assets: &'a dyn I18nAssets,
}

impl Debug for DefaultLocalizer<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "DefaultLocalizer(language_loader: {:p}, i18n_assets: {:p})",
            self.language_loader, self.i18n_assets,
        )
    }
}

#[allow(single_use_lifetimes)]
impl<'a> Localizer for DefaultLocalizer<'a> {
    fn language_loader(&self) -> &'_ dyn LanguageLoader {
        self.language_loader
    }
    fn i18n_assets(&self) -> &'_ dyn I18nAssets {
        self.i18n_assets
    }
}

impl<'a> DefaultLocalizer<'a> {
    /// Create a new [DefaultLocalizer](DefaultLocalizer).
    pub fn new(
        language_loader: &'a dyn LanguageLoader,
        i18n_assets: &'a dyn I18nAssets,
    ) -> DefaultLocalizer<'a> {
        DefaultLocalizer {
            language_loader,
            i18n_assets,
        }
    }
}

/// Select the most suitable available language in order of preference
/// by `requested_languages`, and load it using the provided
/// [LanguageLoader] from the languages available in [I18nAssets].
/// Returns the available languages that were negotiated as being the
/// most suitable to be selected, and were loaded by
/// [LanguageLoader::load_languages()]. If there were no available
/// languages, then no languages will be loaded and the returned
/// `Vec` will be empty.
pub fn select(
    language_loader: &dyn LanguageLoader,
    i18n_assets: &dyn I18nAssets,
    requested_languages: &[unic_langid::LanguageIdentifier],
) -> Result<Vec<unic_langid::LanguageIdentifier>, I18nEmbedError> {
    debug!(
        "Selecting translations for domain \"{0}\"",
        language_loader.domain()
    );

    let available_languages: Vec<unic_langid::LanguageIdentifier> =
        language_loader.available_languages(i18n_assets)?;
    let default_language: &unic_langid::LanguageIdentifier = language_loader.fallback_language();

    let supported_languages = negotiate_languages(
        requested_languages,
        &available_languages,
        Some(default_language),
        NegotiationStrategy::Filtering,
    );

    debug!("Requested Languages: {:?}", requested_languages);
    debug!("Available Languages: {:?}", available_languages);
    debug!("Supported Languages: {:?}", supported_languages);

    if !supported_languages.is_empty() {
        language_loader.load_languages(i18n_assets, supported_languages.as_slice())?;
    }

    Ok(supported_languages.into_iter().cloned().collect())
}

/// A language resource file, and its associated `language`.
#[derive(Debug)]
pub struct LanguageResource<'a> {
    /// The language which this resource is associated with.
    pub language: unic_langid::LanguageIdentifier,
    /// The data for the file containing the localizations.
    pub file: Cow<'a, [u8]>,
}

/// A trait used by [I18nAssets](I18nAssets) to load a language file for
/// a specific rust module using a specific localization system. The
/// trait is designed such that the loader could be swapped during
/// runtime, or contain state if required.
pub trait LanguageLoader {
    /// The fallback language for the module this loader is responsible
    /// for.
    fn fallback_language(&self) -> &unic_langid::LanguageIdentifier;
    /// The domain for the translation that this loader is associated with.
    fn domain(&self) -> &str;
    /// The language file name to use for this loader's domain.
    fn language_file_name(&self) -> String;
    /// The computed path to the language file, and `Cow` of the file
    /// itself if it exists.
    fn language_file<'a>(
        &self,
        language_id: &unic_langid::LanguageIdentifier,
        i18n_assets: &'a dyn I18nAssets,
    ) -> (String, Option<Cow<'a, [u8]>>) {
        let language_id_string = language_id.to_string();
        let file_path = format!("{}/{}", language_id_string, self.language_file_name());

        log::debug!("Attempting to load language file: \"{}\"", &file_path);

        let file = i18n_assets.get_file(file_path.as_ref());
        (file_path, file)
    }

    /// Calculate the languages which are available to be loaded.
    fn available_languages(
        &self,
        i18n_assets: &dyn I18nAssets,
    ) -> Result<Vec<unic_langid::LanguageIdentifier>, I18nEmbedError> {
        let mut language_strings: Vec<String> = i18n_assets
            .filenames_iter()
            .filter_map(|filename| {
                let path: &Path = Path::new(&filename);

                let components: Vec<Component<'_>> = path.components().collect();

                let locale: Option<String> = match components.get(0) {
                    Some(Component::Normal(s)) => {
                        Some(s.to_str().expect("path should be valid utf-8").to_string())
                    }
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
                        debug!(
                            "Searching for available languages, found language file: \"{0}\"",
                            &filename
                        );
                        if language_file_name == self.language_file_name() {
                            locale
                        } else {
                            None
                        }
                    }
                    None => None,
                }
            })
            .collect();

        let fallback_locale = self.fallback_language().to_string();

        // For systems such as gettext which have a locale in the
        // source code, this language will not be found in the
        // localization assets, and should be the fallback_locale, so
        // it needs to be added manually here.
        if !language_strings
            .iter()
            .any(|language| language == &fallback_locale)
        {
            language_strings.insert(0, fallback_locale);
        }

        language_strings
            .into_iter()
            .map(|language: String| {
                language
                    .parse()
                    .map_err(|err| I18nEmbedError::ErrorParsingLocale(language, err))
            })
            .collect()
    }

    /// Get the language which is currently loaded for this loader.
    fn current_language(&self) -> unic_langid::LanguageIdentifier;

    /// Load the languages `language_ids` using the resources packaged
    /// in the `i18n_embed` in order of fallback preference. This also
    /// sets the [LanguageLoader::current_language()] to the first in
    /// the `language_ids` slice. You can use [select()] to determine
    /// which fallbacks are actually available for an arbitrary slice
    /// of preferences.
    fn load_languages(
        &self,
        i18n_assets: &dyn I18nAssets,
        language_ids: &[&unic_langid::LanguageIdentifier],
    ) -> Result<(), I18nEmbedError>;

    /// Load the [LanguageLoader::fallback_language()].
    fn load_fallback_language(&self, i18n_assets: &dyn I18nAssets) -> Result<(), I18nEmbedError> {
        self.load_languages(i18n_assets, &[self.fallback_language()])
    }
}

/// Populate gettext database with strings for use with tests.
#[cfg(all(test, feature = "gettext-system"))]
mod gettext_test_string {
    fn _test_strings() {
        tr::tr!("only en");
        tr::tr!("only ru");
        tr::tr!("only es");
        tr::tr!("only fr");
    }
}
