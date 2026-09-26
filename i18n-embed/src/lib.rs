#![doc = include_str!("../README.md")]
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
#![cfg_attr(docsrs, feature(doc_cfg))]

mod assets;
mod requester;
mod util;

#[cfg(feature = "fluent-system")]
#[cfg_attr(docsrs, doc(cfg(feature = "fluent-system")))]
pub mod fluent;

#[cfg(feature = "gettext-system")]
#[cfg_attr(docsrs, doc(cfg(feature = "gettext-system")))]
pub mod gettext;

pub use assets::*;
pub use requester::*;
pub use util::*;

use std::{
    borrow::Cow,
    fmt::Debug,
    path::{Component, Path},
    string::FromUtf8Error,
};

use fluent_langneg::{NegotiationStrategy, negotiate_languages};
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
    #[cfg_attr(docsrs, doc(cfg(feature = "gettext-system")))]
    #[error(transparent)]
    Gettext(#[from] ::gettext::Error),
    #[cfg(feature = "autoreload")]
    #[cfg_attr(docsrs, doc(cfg(feature = "autoreload")))]
    #[error(transparent)]
    Notify(#[from] assets::NotifyError),
    #[cfg(feature = "filesystem-assets")]
    #[cfg_attr(docsrs, doc(cfg(feature = "filesystem-assets")))]
    #[error("The directory {0:?} does not exist")]
    DirectoryDoesNotExist(std::path::PathBuf),
    #[cfg(feature = "filesystem-assets")]
    #[cfg_attr(docsrs, doc(cfg(feature = "filesystem-assets")))]
    #[error("The path {0:?} is not a directory")]
    PathIsNotDirectory(std::path::PathBuf),
}

fn error_vec_to_string(errors: &[I18nEmbedError]) -> String {
    let strings: Vec<String> = errors.iter().map(|e| format!("{e}")).collect();
    strings.join(", ")
}

/// This trait provides dynamic access to a [`LanguageLoader`] and an [`I18nAssets`],
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

    /// Select the requested languages and load them using the provided [`LanguageLoader`].
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

/// A simple default implementation of the [`Localizer`] trait.
pub struct DefaultLocalizer<'a> {
    /// The source of assets used by this localizer.
    pub i18n_assets: &'a (dyn I18nAssets + Send + Sync + 'static),
    /// The [LanguageLoader] used by this localizer.
    pub language_loader: &'a (dyn LanguageLoader + Send + Sync + 'static),
    watchers: Vec<Box<dyn Watcher + Send + Sync + 'static>>,
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
    fn i18n_assets(&self) -> &'_ dyn I18nAssets {
        self.i18n_assets
    }
    fn language_loader(&self) -> &'_ dyn LanguageLoader {
        self.language_loader
    }
}

impl<'a> DefaultLocalizer<'a> {
    /// Create a new [`DefaultLocalizer`].
    pub fn new(
        language_loader: &'a (dyn LanguageLoader + Send + Sync + 'static),
        i18n_assets: &'a (dyn I18nAssets + Send + Sync + 'static),
    ) -> Self {
        Self {
            i18n_assets,
            language_loader,
            watchers: Vec::new(),
        }
    }
}

impl DefaultLocalizer<'static> {
    /// Create a new [`DefaultLocalizer`] which will attempt to automatically reload changed assets.
    ///
    /// This will do nothing unless the `autoreload` crate feature is enabled.
    pub fn with_autoreload(mut self) -> Result<Self, I18nEmbedError> {
        let assets = self.i18n_assets;
        let loader = self.language_loader;
        let watcher = self
            .i18n_assets
            .subscribe_changed(std::sync::Arc::new(move || {
                if let Err(error) = loader.reload(assets) {
                    error!("Error autoreloading assets: {error:?}")
                }
            }))?;
        self.watchers.push(watcher);
        Ok(self)
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
    log::info!(
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

    log::debug!("Requested Languages: {:?}", requested_languages);
    log::debug!("Available Languages: {:?}", available_languages);
    log::debug!("Supported Languages: {:?}", supported_languages);

    let supported_languages: Vec<unic_langid::LanguageIdentifier> =
        supported_languages.into_iter().cloned().collect();
    if !supported_languages.is_empty() {
        language_loader.load_languages(i18n_assets, &supported_languages)?;
    }

    Ok(supported_languages)
}

/// A language resource file, and its associated `language`.
#[derive(Debug)]
pub struct LanguageResource<'a> {
    /// The language which this resource is associated with.
    pub language: unic_langid::LanguageIdentifier,
    /// The data for the file containing the localizations.
    pub file: Cow<'a, [u8]>,
}

/// A trait used by [`I18nAssets`] to load a language file for
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
    /// The computed path to the language files, and data contained within the files at that path
    /// itself if they exist. There can be multiple files at a given path, in order of preference
    /// from high to low.
    fn language_files<'a>(
        &self,
        language_id: &unic_langid::LanguageIdentifier,
        i18n_assets: &'a dyn I18nAssets,
    ) -> (String, Vec<Cow<'a, [u8]>>) {
        let language_id_string = language_id.to_string();
        let file_path = format!("{}/{}", language_id_string, self.language_file_name());

        debug!("Attempting to load language file: \"{}\"", file_path);

        let files = i18n_assets.get_files(file_path.as_ref());
        (file_path, files)
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

                let locale: Option<String> = match components.first() {
                    Some(Component::Normal(s)) => {
                        Some(s.to_str().expect("path should be valid utf-8").to_string())
                    }
                    _ => None,
                };

                let language_file_name: Option<String> =
                    components.get(1).and_then(|component| match component {
                        Component::Normal(s) => {
                            Some(s.to_str().expect("path should be valid utf-8").to_string())
                        }
                        _ => None,
                    });

                match language_file_name {
                    Some(language_file_name) => {
                        debug!(
                            "Searching for available languages, found language file: \"{0}\"",
                            filename
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

    /// Load all available languages with [`LanguageLoader::load_languages()`].
    fn load_available_languages(&self, i18n_assets: &dyn I18nAssets) -> Result<(), I18nEmbedError> {
        let available_languages = self.available_languages(i18n_assets)?;
        self.load_languages(i18n_assets, &available_languages)
    }

    /// Get the language which is currently loaded for this loader.
    fn current_language(&self) -> unic_langid::LanguageIdentifier;

    /// Reload the currently loaded languages.
    fn reload(&self, i18n_assets: &dyn I18nAssets) -> Result<(), I18nEmbedError>;

    /// Load the languages `language_ids` using the resources packaged
    /// in the `i18n_embed` in order of fallback preference. This also
    /// sets the [LanguageLoader::current_language()] to the first in
    /// the `language_ids` slice. You can use [select()] to determine
    /// which fallbacks are actually available for an arbitrary slice
    /// of preferences.
    fn load_languages(
        &self,
        i18n_assets: &dyn I18nAssets,
        language_ids: &[unic_langid::LanguageIdentifier],
    ) -> Result<(), I18nEmbedError>;

    /// Load the [LanguageLoader::fallback_language()].
    fn load_fallback_language(&self, i18n_assets: &dyn I18nAssets) -> Result<(), I18nEmbedError> {
        self.load_languages(i18n_assets, &[self.fallback_language().clone()])
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
