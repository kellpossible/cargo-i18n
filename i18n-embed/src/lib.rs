#[allow(unused_imports)]
#[macro_use]
extern crate i18n_embed_impl;
pub use i18n_embed_impl::*;

use std::io;

use core::fmt::Display;
use fluent_langneg::{negotiate_languages, NegotiationStrategy};
use rust_embed::RustEmbed;
use unic_langid::LanguageIdentifier;

pub trait I18nEmbedLogger {
    fn embed_log<D: Display>(&self, message: D);
}

pub trait LanguageRequester {
    fn requested_languages(&self) -> Vec<LanguageIdentifier>;
}

pub trait LanguageLoader {
    fn module_path() -> &'static str;
    fn load_language_file<R: io::Read>(&self, reader: R);
}

pub trait I18nEmbed: RustEmbed {
    fn src_locale() -> LanguageIdentifier;

    fn language_file_name<L: LanguageLoader>() -> String {
        format!("{}.{}", domain_from_module(L::module_path()), "mo")
    }

    fn available_languages<L: LanguageLoader, D: I18nEmbedLogger>(logger: &D) -> Vec<LanguageIdentifier> {
        use std::path::{Component, Path};

        logger.embed_log(format!("Looking For Language File: {:?}", Self::language_file_name::<L>()));

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

                let language_file_name: Option<String> = components.get(1).map(|component| {
                    match component {
                        Component::Normal(s) => {
                            Some(s.to_str().expect("path should be valid utf-8").to_string())
                        },
                        _ => None,
                    }
                }).flatten();

                logger.embed_log(format!("Language File: {:?}", language_file_name));

                match language_file_name {
                    Some(language_file_name) => {
                        if language_file_name == Self::language_file_name::<L>() {
                            locale
                        } else {
                            None
                        }
                    },
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
                    logger.embed_log(format!("Unable to parse language: {:?}", err));
                    None
                }
            })
            .collect();
    }

    fn test<R: LanguageRequester>(language_requester: &R) {
        println!("{:?}", language_requester.requested_languages());
    }

    fn custom_select<R: LanguageRequester, L: LanguageLoader, D: I18nEmbedLogger>(
        language_requester: &R,
        language_loader: &L,
        logger: &D,
    ) {
        logger.embed_log(format!(
            "Available Languages: {:?}",
            Self::available_languages::<L, D>(logger)
        ));

        let requested_languages = language_requester.requested_languages();

        let available_languages: Vec<LanguageIdentifier> = Self::available_languages::<L, D>(logger);
        let default_language: LanguageIdentifier = Self::src_locale();

        let supported_languages = negotiate_languages(
            &requested_languages,
            &available_languages,
            Some(&default_language),
            NegotiationStrategy::Filtering,
        );

        logger.embed_log(format!("Requested Languages: {:?}", requested_languages));
        logger.embed_log(format!("Available Languages: {:?}", available_languages));
        logger.embed_log(format!("Supported Languages: {:?}", supported_languages));

        match supported_languages.get(0) {
            Some(language_id) => {
                if language_id != &&default_language {
                    let language_id_string = language_id.to_string();
                    let f = Self::get(
                        format!("{}/{}", language_id_string, Self::language_file_name::<L>()).as_ref(),
                    )
                    .expect("could not read the file");
                    language_loader.load_language_file(&*f);
                }
            }
            None => {
                // do nothing
            }
        }

        logger.embed_log("Completed setting translations!");
    }

    #[cfg(feature = "desktop-requester")]
    fn select<L: LanguageLoader>(loader: &L) {
        let requester = DesktopLanguageRequester::new();
        let logger = DesktopLogger::new();
        Self::custom_select(&requester, loader, &logger);
    }

    #[cfg(feature = "web-sys-requester")]
    fn select() {}
}

#[cfg(feature = "desktop-requester")]
pub struct DesktopLanguageRequester;

#[cfg(feature = "desktop-requester")]
pub struct DesktopLogger;

#[cfg(feature = "desktop-requester")]
impl I18nEmbedLogger for DesktopLogger {
    fn embed_log<D: Display>(&self, message: D) {
        println!("{}", message);
    }
}

#[cfg(feature = "desktop-requester")]
impl DesktopLogger {
    pub fn new() -> DesktopLogger {
        DesktopLogger {}
    }
}

#[cfg(feature = "web-sys-requester")]
struct WebSysLogger;

#[cfg(feature = "web-sys-requester")]
impl WebSysLogger {
    pub fn new() -> WebSysLogger {
        WebSysLogger {}
    }
}

#[cfg(feature = "web-sys-requester")]
impl I18nEmbedLogger for WebSysLogger {
    fn embed_log<D: Display>(&self, message: D) {
        use web_sys::console;
        console::log_1(&format!("{}", message).into());
    }
}

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
                    eprintln!("Unable to parse your locale: {:?}", err);
                    None
                }
            })
            .collect();

        println!("Current Locale: {:?}", ids);

        ids
    }
}

#[cfg(feature = "web-sys-requester")]
pub struct WebLanguageRequester;

#[cfg(feature = "web-sys-requester")]
impl LanguageRequester for WebLanguageRequester {
    fn requested_languages(&self) -> Vec<LanguageIdentifier> {
        use web_sys;
        let window = web_sys::window().expect("no global `window` exists");
        let navigator = window.navigator();
        let languages = navigator.languages();

        let requested_languages =
            convert_vec_str_to_langids_lossy(languages.iter().map(|js_value| {
                js_value
                    .as_string()
                    .expect("language value should be a string.")
            }));
    }
}

pub fn domain_from_module(module: &str) -> &str {
    module.split("::").next().unwrap_or(module)
}
