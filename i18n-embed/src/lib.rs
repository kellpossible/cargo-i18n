#![allow(clippy::needless_doctest_main)]
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
//!     desktop platform (windows, mac, linux), which makes use of the
//!     [locale_config](https://crates.io/crates/locale_config) crate
//!     for resolving the current system locale.
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
//! ### Simple
//!
//! The following is an example for how to derive the required traits
//! on structs, and localize your binary using this library when it
//! first runs:
//!
//! ```
//! use i18n_embed::{I18nEmbed, language_loader, DesktopLanguageRequester};
//! use rust_embed::RustEmbed;
//!
//! #[derive(RustEmbed, I18nEmbed)]
//! #[folder = "i18n/mo"] // path to the compiled localization resources
//! struct Translations;
//!
//! language_loader!(MyLanguageLoader);
//!
//! fn main() {
//!     let translations = Translations {};
//!     let language_loader = MyLanguageLoader::new();
//!
//!     // Use the language requester for the desktop platform (linux, windows, mac).
//!     // There is also a requester available for the web-sys WASM platform called
//!     // WebLanguageRequester, or you can implement your own.
//!     let requested_languages = DesktopLanguageRequester::requested_languages();
//!
//!     i18n_embed::select(&language_loader, &translations, &requested_languages);
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
//! use std::rc::Rc;
//! use i18n_embed::{
//!     I18nEmbed, language_loader, DesktopLanguageRequester,
//!     LanguageRequester, DefaultLocalizer, Localizer};
//! use rust_embed::RustEmbed;
//! use lazy_static::lazy_static;
//!
//! #[derive(RustEmbed, I18nEmbed)]
//! #[folder = "i18n/mo"] // path to the compiled localization resources
//! struct Translations;
//! const TRANSLATIONS: Translations = Translations {};
//!
//! language_loader!(MyLanguageLoader);
//!
//! lazy_static! {
//!     static ref LANGUAGE_LOADER: MyLanguageLoader = MyLanguageLoader::new();
//! }
//!
//! fn main() {
//!     let localizer = DefaultLocalizer::new(
//!         &*LANGUAGE_LOADER,
//!         &TRANSLATIONS,
//!     );
//!
//!     let localizer_rc: Rc<dyn Localizer> = Rc::new(localizer);
//!
//!     let mut language_requester = DesktopLanguageRequester::new();
//!     language_requester.add_listener(&localizer_rc);
//!
//!     // Manually check the currently requested system language,
//!     // and update the listeners. When the system language changes,
//!     // this will automatically be triggered.
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
//! # #![feature(proc_macro_hygiene)]
//! use std::rc::Rc;
//! use i18n_embed::{
//!    I18nEmbed, language_loader, DesktopLanguageRequester,
//!    LanguageRequester, DefaultLocalizer, Localizer};
//! use rust_embed::RustEmbed;
//! use lazy_static::lazy_static;
//!
//! #[derive(RustEmbed, I18nEmbed)]
//! #[folder = "i18n/mo"] // path to the compiled localization resources
//! struct Translations;
//! const TRANSLATIONS: Translations = Translations {};
//!
//! language_loader!(MyLanguageLoader);
//!
//! lazy_static! {
//!     static ref LANGUAGE_LOADER: MyLanguageLoader = MyLanguageLoader::new();
//! }
//!
//! // Get the `Localizer` to be used for localizing this library.
//! #[cfg(feature = "localize")]
//! pub fn localizer() -> Box<dyn Localizer<'static>> {
//!     Box::from(DefaultLocalizer::new(
//!         &LANGUAGE_LOADER,
//!         &TRANSLATIONS
//!     ))
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
//! # #![feature(proc_macro_hygiene)]
//! use std::rc::Rc;
//! use i18n_embed::{
//!    I18nEmbed, language_loader, DesktopLanguageRequester,
//!    LanguageRequester, DefaultLocalizer, Localizer};
//! use rust_embed::RustEmbed;
//! use i18n_embed::I18nEmbedDyn;
//! use lazy_static::lazy_static;
//!
//! #[derive(RustEmbed, I18nEmbed)]
//! #[folder = "i18n/mo"] // path to the compiled localization resources
//! struct Translations;
//!
//! const TRANSLATIONS: Translations = Translations {};
//!
//! language_loader!(MyLanguageLoader);
//!
//! lazy_static! {
//!     static ref LANGUAGE_LOADER: MyLanguageLoader = MyLanguageLoader::new();
//! }
//!
//! // Get the `Localizer` to be used for localizing this library,
//! // using the provided embeddes source of language files `embed`.
//! pub fn localizer(embed: &'static dyn I18nEmbedDyn) -> Box<dyn Localizer<'static>> {
//!     Box::from(DefaultLocalizer::new(
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
use std::rc::{Rc, Weak};

use fluent_langneg::{negotiate_languages, NegotiationStrategy};
use log::{debug, error, info};
use rust_embed::RustEmbed;
use thiserror::Error;

pub use gettext;
pub use tr;
pub use unic_langid;

/// An error that occurs in this library.
#[derive(Error, Debug)]
pub enum I18nEmbedError {
    #[error("Error parsing a language identifier string \"{0}\"")]
    ErrorParsingLocale(String, #[source] unic_langid::LanguageIdentifierError),
    #[error("The requested language \"{0}\" is not available.")]
    LanguageNotAvailable(String),
    #[error("There are multiple errors: {}", error_vec_to_string(.0))]
    Multiple(Vec<I18nEmbedError>),
}

fn error_vec_to_string(errors: &[I18nEmbedError]) -> String {
    let strings: Vec<String> = errors.iter().map(|e| format!("{}", e)).collect();
    strings.join(", ")
}

/// This trait provides dynamic access to an
/// [LanguageLoader](LanguageLoader) and an [I18nEmbed](I18nEmbed),
/// which are used together to localize a library/crate on demand.
pub trait Localizer<'a> {
    fn language_loader(&self) -> &'a dyn LanguageLoader;
    fn i18n_embed(&self) -> &'a dyn I18nEmbedDyn;

    fn available_languages(&self) -> Result<Vec<unic_langid::LanguageIdentifier>, I18nEmbedError> {
        self.i18n_embed()
            .available_languages_dyn(self.language_loader())
    }

    /// Automatically the language currently requested by the system
    /// by the the [LanguageRequester](LanguageRequester)), and load
    /// it using the provided [LanguageLoader](LanguageLoader).
    fn select(
        &self,
        requested_languages: &[unic_langid::LanguageIdentifier],
    ) -> Result<Option<unic_langid::LanguageIdentifier>, I18nEmbedError> {
        select(
            self.language_loader(),
            self.i18n_embed(),
            requested_languages,
        )
    }
}

/// A simple default implemenation of the [Localizer](Localizer) trait.
pub struct DefaultLocalizer<'a> {
    pub language_loader: &'a dyn LanguageLoader,
    pub i18n_embed: &'a dyn I18nEmbedDyn,
}

impl<'a> Localizer<'a> for DefaultLocalizer<'a> {
    fn language_loader(&self) -> &'a dyn LanguageLoader {
        self.language_loader
    }
    fn i18n_embed(&self) -> &'a dyn I18nEmbedDyn {
        self.i18n_embed
    }
}

impl<'a> DefaultLocalizer<'a> {
    /// Create a new [DefaultLocalizer](DefaultLocalizer).
    pub fn new(
        language_loader: &'a dyn LanguageLoader,
        i18n_embed: &'a dyn I18nEmbedDyn,
    ) -> DefaultLocalizer<'a> {
        DefaultLocalizer {
            language_loader,
            i18n_embed,
        }
    }
}

/// Provide the functionality for overrides and listeners for a
/// [LanguageRequester](LanguageReqeuster) implementation.
struct LanguageRequesterImpl<'a> {
    listeners: Vec<Weak<dyn Localizer<'a>>>,
    language_override: Option<unic_langid::LanguageIdentifier>,
}

impl<'a> LanguageRequesterImpl<'a> {
    /// Create a new [LanguageRequesterImpl](LanguageRequesterImpl).
    pub fn new() -> LanguageRequesterImpl<'a> {
        LanguageRequesterImpl {
            listeners: Vec::new(),
            language_override: None,
        }
    }

    /// Set an override for the requested language which is used when the
    /// [LanguageRequesterImpl#poll()](LanguageRequester#poll()) method
    /// is called. If `None`, then no override is used.
    fn set_languge_override(
        &mut self,
        language_override: Option<unic_langid::LanguageIdentifier>,
    ) -> Result<(), I18nEmbedError> {
        self.language_override = language_override;
        Ok(())
    }

    fn add_listener(&mut self, localizer: &Rc<dyn Localizer<'a>>) {
        self.listeners.push(Rc::downgrade(localizer));
    }

    /// With the provided `requested_languages` call
    /// [Localizer#select()](Localizer#select()) on each of the
    /// listeners.
    fn poll_without_override(
        &mut self,
        requested_languages: Vec<unic_langid::LanguageIdentifier>,
    ) -> Result<(), I18nEmbedError> {
        let mut errors: Vec<I18nEmbedError> = Vec::new();

        self.listeners.retain(|listener| match listener.upgrade() {
            Some(l) => {
                match l.select(&requested_languages) {
                    Ok(_) => {}
                    Err(err) => {
                        errors.push(err);
                    }
                }

                true
            }
            None => false,
        });

        if errors.is_empty() {
            Ok(())
        } else if errors.len() == 1 {
            Err(errors.into_iter().next().unwrap())
        } else {
            Err(I18nEmbedError::Multiple(errors))
        }
    }

    /// With the provided `requested_languages` call
    /// [Localizer#select()](Localizer#select()) on each of the
    /// listeners. The `requested_languages` may be ignored if
    /// [#set_languge_override()](#set_languge_override()) has been
    /// set.
    pub fn poll(
        &mut self,
        requested_languages: Vec<unic_langid::LanguageIdentifier>,
    ) -> Result<(), I18nEmbedError> {
        let languages = match &self.language_override {
            Some(language) => {
                debug!("Using language override: {}", language);
                vec![language.clone()]
            }
            None => requested_languages,
        };

        self.poll_without_override(languages)
    }
}

/// A trait used by [I18nEmbed](I18nEmbed) to ascertain which
/// languages are being requested.
pub trait LanguageRequester<'a> {
    /// Add a listener to this `LanguageRequester`. When the system
    /// reports that the currently requested languages has changed,
    /// each listener will have its
    /// [Localizer#select()](Localizer#select()) method called.
    ///
    /// If you haven't already selected a language for the localizer
    /// you are adding here, you may want to manually call
    /// [#poll()](#poll()) after adding the listener/s.
    fn add_listener(&mut self, localizer: &Rc<dyn Localizer<'a>>);
    /// Poll the system's currently selected language, and call
    /// [Localizer#select()](Localizer#select()) on each of the
    /// listeners.
    fn poll(&mut self) -> Result<(), I18nEmbedError>;
    /// Override the languages fed to the [Localizer](Localizer) listeners during
    /// a [#poll()](#poll()). Set this as `None` to disable the override.
    fn set_languge_override(
        &mut self,
        language_override: Option<unic_langid::LanguageIdentifier>,
    ) -> Result<(), I18nEmbedError>;
    /// Get the language last selected for the [LanguageLoader](LanguageLoader)
    /// by this [LanguageRequester](LanguageRequester).
    // fn currently_selected_language(&self, language_loader: &'a dyn LanguageLoader) -> &Option<unic_langid::LanguageIdentifier>;
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

/// Select the most suitable language currently requested by the
/// system by the the [LanguageRequester](LanguageRequester), and load
/// it using the provided [LanguageLoader](LanguageLoader) from the
/// languages embedded in [I18nEmbed](I18nEmbed) via
/// [I18nEmbedDyn](I18nEmbedDyn). Returns the language that was
/// negotiated to be selected.
pub fn select(
    language_loader: &dyn LanguageLoader,
    i18n_embed: &dyn I18nEmbedDyn,
    requested_languages: &[unic_langid::LanguageIdentifier],
) -> Result<Option<unic_langid::LanguageIdentifier>, I18nEmbedError> {
    info!(
        "Selecting translations for domain \"{0}\"",
        language_loader.domain()
    );

    let available_languages: Vec<unic_langid::LanguageIdentifier> =
        i18n_embed.available_languages_dyn(language_loader)?;
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
            Ok(Some((*language_id).clone()))
        }
        None => Ok(None),
    }
}

/// Load a language with `language_id` using the provided
/// [LanguageLoader](LanguageLoader) from the languages embedded in
/// [I18nEmbed](I18nEmbed) via [I18nEmbedDyn](I18nEmbedDyn).
pub fn select_single(
    language_loader: &dyn LanguageLoader,
    i18n_embed: &dyn I18nEmbedDyn,
    language_id: &unic_langid::LanguageIdentifier,
) -> Result<(), I18nEmbedError> {
    language_loader.load_language(language_id, i18n_embed)?;
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
    /// The domain for the translation that this loader is associated with.
    fn domain(&self) -> &'static str;
    /// Load the language corresponding the specified `language_id`.
    fn load_language(
        &self,
        language_id: &unic_langid::LanguageIdentifier,
        i18n_embed: &dyn I18nEmbedDyn,
    ) -> Result<(), I18nEmbedError> {
        if language_id == &self.src_locale() {
            self.load_src_locale();
            return Ok(());
        }
        let language_id_string = language_id.to_string();

        let file_path = format!("{}/{}", language_id_string, self.language_file_name());

        let f = i18n_embed
            .get_dyn(file_path.as_ref())
            .ok_or(I18nEmbedError::LanguageNotAvailable(language_id_string))?;
        info!(
            "Loading language \"{0}\" from file \"{1}\"",
            language_id.to_string(),
            file_path
        );
        self.load_language_file(language_id.clone(), f);

        Ok(())
    }
    /// Load the language `file` which is associated with the specified `language_id` and
    /// set the current language, which can be retrieved via
    /// [current_language()](LanguageLoader#current_language()).
    fn load_language_file(&self, language_id: unic_langid::LanguageIdentifier, file: Cow<[u8]>);
    /// Load the language associated with [src_locale()](LanguageLoader#src_locale()).
    fn load_src_locale(&self);
    /// The language file name to use for this loader.
    fn language_file_name(&self) -> String;
    /// Get the language which is currently loaded for this loader.
    fn current_language(&self) -> unic_langid::LanguageIdentifier;
}

/// A dynamic reference to a static [I18nEmbed](I18nEmbed) implementation.
pub trait I18nEmbedDyn {
    /// A dynamic way to access the static
    /// [I18nEmbed#available_languages()](I18nEmbed#available_languages())
    /// method for a given [I18nEmbed](I18nEmbed) implementation.
    fn available_languages_dyn<'a>(
        &self,
        language_loader: &'a dyn LanguageLoader,
    ) -> Result<Vec<unic_langid::LanguageIdentifier>, I18nEmbedError>;
    /// A dynamic way to access the static [RustEmbed#get()](RustEmbed#get())
    /// for a given [I18nEmbed](I18nEmbed) implementation.
    fn get_dyn(&self, file_path: &str) -> Option<std::borrow::Cow<'static, [u8]>>;
}

impl<T: I18nEmbed + ?Sized> I18nEmbedDyn for T {
    fn available_languages_dyn<'a>(
        &self,
        language_loader: &'a dyn LanguageLoader,
    ) -> Result<Vec<unic_langid::LanguageIdentifier>, I18nEmbedError> {
        T::available_languages(language_loader)
    }
    fn get_dyn(&self, file_path: &str) -> Option<std::borrow::Cow<'static, [u8]>> {
        T::get(file_path)
    }
}

/// A trait to handle the embedding of software translations within
/// the current binary, and the retrieval/loading of those
/// translations at runtime.
pub trait I18nEmbed: RustEmbed {
    /// Calculate the embedded languages available to be selected for
    /// the module requested by the provided [LanguageLoader](LanguageLoader).
    fn available_languages(
        language_loader: &'_ dyn LanguageLoader,
    ) -> Result<Vec<unic_langid::LanguageIdentifier>, I18nEmbedError> {
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
                        debug!(
                            "Searching for available languages, found language file: \"{0}\"",
                            &filename
                        );
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
            .map(|language: String| {
                language
                    .parse()
                    .map_err(|err| I18nEmbedError::ErrorParsingLocale(language, err))
            })
            .collect()
    }
}

/// A [LanguageRequester](LanguageRequester) for the desktop platform,
/// supporting windows, linux and mac. It uses
/// [locale_config](locale_config) to select the language based on the
/// system selected language.
///
/// ⚠️ *This API requires the following crate features to be activated: `desktop-requester`.*
#[cfg(feature = "desktop-requester")]
pub struct DesktopLanguageRequester<'a> {
    implementation: LanguageRequesterImpl<'a>,
}

#[cfg(feature = "desktop-requester")]
impl<'a> LanguageRequester<'a> for DesktopLanguageRequester<'a> {
    fn requested_languages(&self) -> Vec<unic_langid::LanguageIdentifier> {
        Self::requested_languages()
    }

    fn add_listener(&mut self, localizer: &Rc<dyn Localizer<'a>>) {
        self.implementation.add_listener(localizer)
    }

    fn set_languge_override(
        &mut self,
        language_override: Option<unic_langid::LanguageIdentifier>,
    ) -> Result<(), I18nEmbedError> {
        self.implementation.set_languge_override(language_override)
    }

    fn poll(&mut self) -> Result<(), I18nEmbedError> {
        self.implementation.poll(self.requested_languages())
    }
}

#[cfg(feature = "desktop-requester")]
impl<'a> Default for DesktopLanguageRequester<'a> {
    fn default() -> Self {
        DesktopLanguageRequester::new()
    }
}

#[cfg(feature = "desktop-requester")]
impl<'a> DesktopLanguageRequester<'a> {
    pub fn new() -> DesktopLanguageRequester<'a> {
        DesktopLanguageRequester {
            implementation: LanguageRequesterImpl::new(),
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
///
/// ⚠️ *This API requires the following crate features to be activated: `web-sys-requester`.*
#[cfg(feature = "web-sys-requester")]
pub struct WebLanguageRequester<'a> {
    implementation: LanguageRequesterImpl<'a>,
}

#[cfg(feature = "web-sys-requester")]
impl<'a> WebLanguageRequester<'a> {
    pub fn new() -> WebLanguageRequester<'a> {
        WebLanguageRequester {
            implementation: LanguageRequesterImpl::new(),
        }
    }

    pub fn requested_languages() -> Vec<unic_langid::LanguageIdentifier> {
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

#[cfg(feature = "web-sys-requester")]
impl<'a> LanguageRequester<'a> for WebLanguageRequester<'a> {
    fn requested_languages(&self) -> Vec<unic_langid::LanguageIdentifier> {
        Self::requested_languages()
    }

    fn add_listener(&mut self, localizer: &Rc<dyn Localizer<'a>>) {
        self.implementation.add_listener(localizer)
    }

    fn poll(&mut self) -> Result<(), I18nEmbedError> {
        self.implementation.poll(self.requested_languages())
    }

    fn set_languge_override(
        &mut self,
        language_override: Option<unic_langid::LanguageIdentifier>,
    ) -> Result<(), I18nEmbedError> {
        self.implementation.set_languge_override(language_override)
    }
}

/// Get the translation domain from the module path (first module in
/// the module path).
pub fn domain_from_module(module_path: &str) -> &str {
    module_path.split("::").next().unwrap_or(module_path)
}
