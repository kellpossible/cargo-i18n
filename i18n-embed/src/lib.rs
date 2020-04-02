#[allow(unused_imports)]
#[macro_use]
extern crate i18n_embed_impl;
pub use i18n_embed_impl::*;

use std::io;

use rust_embed::RustEmbed;
use unic_langid::LanguageIdentifier;
use fluent_langneg::{convert_vec_str_to_langids_lossy, negotiate_languages, NegotiationStrategy};
use core::fmt::Display;

pub trait I18nEmbedLogger {
    fn debug_log<D: Display>(&self, message: D);
}

pub trait LanguageRequester {
    fn requested_languages(&self) -> Vec<LanguageIdentifier>;
}

pub trait LanguageLoader {
    fn load_language_file<R: io::Read>(&self, reader: R);
}

pub trait I18nEmbed: RustEmbed {
    fn default_language() -> LanguageIdentifier;

    fn language_file_name() -> &'static str;

    fn available_languages() -> Vec<String> {
        use std::path::{Path, Component};
        use std::collections::HashSet;

        let mut languages: Vec<String> = Self::iter()
            .map(|filename_cow| filename_cow.to_string())
            .filter_map(|filename| {
                let path: &Path = Path::new(&filename);
    
                let components: Vec<Component> = path
                    .components()
                    .collect();
    
                let component: Option<String> = match components.get(0) {
                    Some(component) => {
                        match component {
                            Component::Normal(s) => {
                                Some(s.to_str().expect("path should be valid utf-8").to_string())
                            },
                            _ => None,
                        }
                    }
                    _ => None,
                };
    
                component
            })
            .collect();

        let mut uniques = HashSet::new();

        languages.retain(|e| uniques.insert(e.clone()));
    
        languages.insert(0, String::from("en-US"));
        return languages;
    }

    fn test<R: LanguageRequester>(language_requester: &R) {
        println!("{:?}", language_requester.requested_languages());
    }

    fn select<R: LanguageRequester, L: LanguageLoader, D: I18nEmbedLogger>(language_requester: &R, language_loader: &L, logger: &D) 
    {
        logger.debug_log(format!("Available Languages: {:?}", Self::available_languages()));

        let requested_languages = language_requester.requested_languages();
        
        let available_languages: Vec<LanguageIdentifier> = convert_vec_str_to_langids_lossy(Self::available_languages());
        let default_language: LanguageIdentifier = Self::default_language();
    
        let supported_languages = negotiate_languages(
            &requested_languages,
            &available_languages,
            Some(&default_language),
            NegotiationStrategy::Filtering,
        );
    
        logger.debug_log(format!("Requested Languages: {:?}", requested_languages));
        logger.debug_log(format!("Available Languages: {:?}", available_languages));
        logger.debug_log(format!("Supported Languages: {:?}", supported_languages));
    
        match supported_languages.get(0) {
            Some(language_id) => {
                if language_id != &&default_language {
                    let language_id_string = language_id.to_string();
                    let f = Self::get(format!("{}/{}", language_id_string, Self::language_file_name()).as_ref())
                        .expect("could not read the file");
                    language_loader.load_language_file(&*f);
                }
            }
            None => {
                // do nothing
            }
        }
    
        logger.debug_log("Completed setting translations!");
    }
}

#[cfg(feature = "desktop-requester")]
pub struct DesktopLanguageRequester;

#[cfg(feature = "desktop-requester")]
pub struct Logger;

#[cfg(feature = "desktop-requester")]
impl I18nEmbedLogger for Logger {
    fn debug_log<D: Display>(&self, message: D) {
        println!("{}", message);
    }
}

#[cfg(feature = "desktop-requester")]
impl Logger {
    pub fn new() -> Logger {
        Logger {}
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
        use locale_config::{Locale, LanguageRange};

        let current_locale = Locale::current();
        
        
        let ids: Vec<LanguageIdentifier> = current_locale.tags_for("messages").filter_map(|tag: LanguageRange| {
            // TODO: perhaps consider adding error handling here
            tag.to_string().parse().ok()
        }).collect();

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

        let requested_languages = convert_vec_str_to_langids_lossy(languages.iter().map(|js_value| {
            js_value
                .as_string()
                .expect("language value should be a string.")
        }));
    }
}