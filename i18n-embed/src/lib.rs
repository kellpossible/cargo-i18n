#[allow(unused_imports)]
#[macro_use]
extern crate i18n_embed_impl;
pub use i18n_embed_impl::*;

use std::io;

use fluent_langneg::{negotiate_languages, NegotiationStrategy};
use log::{debug, error, info};
use rust_embed::RustEmbed;
pub use unic_langid::LanguageIdentifier;
pub use tr::set_translator;
pub use gettext::Catalog;

/// A trait used by [I18nEmbed](I18nEmbed) to ascertain which
/// languages are being requested.
pub trait LanguageRequester {
    fn requested_languages(&self) -> Vec<LanguageIdentifier>;
}

/// A trait used by [I18nEmbed](I18nEmbed) to load a language file for
/// the specified module path.
pub trait LanguageLoader {
    /// The module that this language loader is requesting.
    fn module_path() -> &'static str;
    /// Load the language file corresponding to the module that this
    /// loader requested in `module_path()`.
    fn load_language_file<R: io::Read>(&self, reader: R);
}

/// A trait to handle the embedding of software translations within
/// the current binary, and the retrieval/loading of those
/// translations at runtime.
pub trait I18nEmbed: RustEmbed {
    /// The locale for the project the translations are being embedded
    /// into.
    fn src_locale() -> LanguageIdentifier;

    /// Calculate the language file name to use for the given
    /// [LanguageLoader](LanguageLoader).
    fn language_file_name<L: LanguageLoader>() -> String {
        format!("{}.{}", domain_from_module(L::module_path()), "mo")
    }

    /// Calculate the embedded languages available to be selected for
    /// the module requested by the provided [LanguageLoader](LanguageLoader).
    fn available_languages<L: LanguageLoader>() -> Vec<LanguageIdentifier> {
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
                        if language_file_name == Self::language_file_name::<L>() {
                            locale
                        } else {
                            None
                        }
                    }
                    None => None,
                }
            })
            .collect();

        language_strings.insert(0, Self::src_locale().to_string());

        return language_strings
            .into_iter()
            .filter_map(|language: String| match language.parse() {
                Ok(language) => Some(language),
                Err(err) => {
                    error!(
                        "Unable to parse locale string \"{0}\" because {1:?}",
                        language, err
                    );
                    None
                }
            })
            .collect();
    }

    /// Select the language currently requested by the system by the
    /// the [LanguageRequester](LanguageRequester)), and load it using
    /// the provided [LanguageLoader](LanguageLoader). Logging is
    /// performed using the provided [I18nEmbedLogger](I18nEmbedLogger).
    fn select<R: LanguageRequester, L: LanguageLoader>(
        language_requester: &R,
        language_loader: &L,
    ) {
        info!(
            "Selecting translations for module \"{0}\"",
            L::module_path()
        );
        let requested_languages = language_requester.requested_languages();

        let available_languages: Vec<LanguageIdentifier> = Self::available_languages::<L>();
        let default_language: LanguageIdentifier = Self::src_locale();

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
                if language_id != &&default_language {
                    let language_id_string = language_id.to_string();

                    let file_path = format!("{}/{}", language_id_string, Self::language_file_name::<L>());
                    let f = Self::get(file_path.as_ref())
                        .expect("could not read the file");

                    info!("Selected language {0:?}, loading from file \"{1}\"", language_id, file_path);
                    language_loader.load_language_file(&*f);
                }
            }
            None => {
                // do nothing
            }
        }
    }
}

/// A [LanguageRequester](LanguageRequester) for the desktop platform,
/// supporting windows, linux and mac. It uses
/// [locale_config](locale_config) to select the language based on the
/// system selected language.
#[cfg(feature = "desktop-requester")]
pub struct DesktopLanguageRequester;

#[cfg(feature = "desktop-requester")]
impl LanguageRequester for DesktopLanguageRequester {
    fn requested_languages(&self) -> Vec<LanguageIdentifier> {
        Self::requested_languages()
    }
}

#[cfg(feature = "desktop-requester")]
impl DesktopLanguageRequester {
    pub fn new() -> DesktopLanguageRequester {
        DesktopLanguageRequester {}
    }

    pub fn requested_languages() -> Vec<LanguageIdentifier> {
        use locale_config::{LanguageRange, Locale};

        let current_locale = Locale::current();

        let ids: Vec<LanguageIdentifier> = current_locale
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
    fn requested_languages(&self) -> Vec<LanguageIdentifier> {
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
