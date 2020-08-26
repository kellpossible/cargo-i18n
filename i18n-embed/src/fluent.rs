//! This module contains the types and functions to interact with the
//! `fluent` localization system.
//!
//! Most important is the [FluentLanguageLoader].
//!
//! ⚠️ *This module requires the following crate features to be activated: `fluent-system`.*

use crate::{I18nAssets, I18nEmbedError, LanguageLoader};

pub use i18n_embed_impl::fluent_language_loader;

use fluent::{concurrent::FluentBundle, FluentMessage, FluentResource, FluentValue};
use fluent_syntax::ast::Pattern;
use parking_lot::RwLock;
use std::{borrow::Cow, collections::HashMap, fmt::Debug, sync::Arc};
use unic_langid::LanguageIdentifier;

lazy_static::lazy_static! {
    static ref CURRENT_LANGUAGE: RwLock<LanguageIdentifier> = {
        let language = LanguageIdentifier::default();
        RwLock::new(language)
    };
}

struct LocaleBundle {
    locale: LanguageIdentifier,
    bundle: FluentBundle<Arc<FluentResource>>,
}

impl LocaleBundle {
    fn new(locale: LanguageIdentifier, resources: Vec<Arc<FluentResource>>) -> Self {
        let mut bundle = FluentBundle::new(&[locale.clone()]);

        for resource in &resources {
            if let Err(errors) = bundle.add_resource(Arc::clone(resource)) {
                errors.iter().for_each(|error | {
                    log::error!(target: "i18n_embed::fluent", "Error while adding resource to bundle: {0:?}.", error);
                })
            }
        }

        Self { locale, bundle }
    }
}

impl Debug for LocaleBundle {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "LocaleBundle(locale: {})", self.locale)
    }
}

#[derive(Debug)]
struct LocaleConfig {
    current_language: LanguageIdentifier,
    locale_bundles: Vec<LocaleBundle>,
}

/// [LanguageLoader] implemenation for the `fluent` localization
/// system. Also provides methods to access localizations which have
/// been loaded.
///
/// ⚠️ *This API requires the following crate features to be activated: `fluent-system`.*
#[derive(Debug)]
pub struct FluentLanguageLoader {
    locale_config: RwLock<LocaleConfig>,
    domain: String,
    fallback_language: unic_langid::LanguageIdentifier,
}

impl FluentLanguageLoader {
    /// Create a new `FluentLanguageLoader`, which loads messages for
    /// the specified `domain`, and relies on the specified
    /// `fallback_language` for any messages that do not exist for the
    /// current language.
    pub fn new<S: Into<String>>(
        domain: S,
        fallback_language: unic_langid::LanguageIdentifier,
    ) -> Self {
        let config = LocaleConfig {
            current_language: fallback_language.clone(),
            locale_bundles: Vec::new(),
        };

        Self {
            locale_config: RwLock::new(config),
            domain: domain.into(),
            fallback_language,
        }
    }

    /// Get a localized message referenced by the `message_id`.
    pub fn get(&self, message_id: &str) -> String {
        self.get_args_concrete(message_id, HashMap::new())
    }

    /// A non-generic version of [FluentLanguageLoader::get_args()].
    pub fn get_args_concrete<'a>(
        &self,
        message_id: &str,
        args: HashMap<&'a str, FluentValue<'a>>,
    ) -> String {
        let config_lock = self.locale_config.read();

        let args = if args.is_empty() { None } else { Some(&args) };

        config_lock.locale_bundles.iter().filter_map(|locale_bundle| {
            locale_bundle
                .bundle
                .get_message(message_id)
                .and_then(|m: FluentMessage<'_>| m.value)
                .map(|pattern: &Pattern<'_>| {
                    let mut errors = Vec::new();
                    let value = locale_bundle.bundle.format_pattern(pattern, args, &mut errors);
                    if !errors.is_empty() {
                        log::error!(
                            target:"i18n_embed::fluent",
                            "Failed to format a message for locale \"{}\" and id \"{}\".\nErrors\n{:?}.",
                            &config_lock.current_language, message_id, errors
                        )
                    }

                    value.into()
                    })
            })
            .next()
            .unwrap_or_else(|| {
                log::error!(
                    target:"i18n_embed::fluent",
                    "Unable to find localization for locale \"{}\" and id \"{}\".",
                    config_lock.current_language,
                    message_id
                );
                format!("No localization for id: \"{}\"", message_id)
            })
    }

    /// Get a localized message referenced by the `message_id`, and
    /// formatted with the specified `args`.
    pub fn get_args<'a, S, V>(&self, id: &str, args: HashMap<S, V>) -> String
    where
        S: Into<Cow<'a, str>> + Clone + 'static,
        V: Into<FluentValue<'a>> + Clone + 'static,
    {
        let mut keys: Vec<Cow<'a, str>> = Vec::new();

        let mut map: HashMap<&str, FluentValue<'_>> = HashMap::with_capacity(args.len());

        let mut values = Vec::new();

        for (key, value) in args.into_iter() {
            keys.push(key.into());
            values.push(value.into());
        }

        for (i, key) in keys.iter().rev().enumerate() {
            let value = values.pop().unwrap_or_else(|| {
                panic!(
                    "expected a value corresponding with key \"{}\" at position {}",
                    key, i
                )
            });

            map.insert(&*key, value);
        }

        self.get_args_concrete(id, map)
    }

    /// Returns true if a message with the specified `message_id` is
    /// available in any of the languages currently loaded (including
    /// the fallback language).
    pub fn has(&self, message_id: &str) -> bool {
        let config_lock = self.locale_config.read();
        let mut has_message = false;

        config_lock
            .locale_bundles
            .iter()
            .for_each(|locale_bundle| has_message |= locale_bundle.bundle.has_message(message_id));

        has_message
    }
}

impl LanguageLoader for FluentLanguageLoader {
    /// The fallback language for the module this loader is responsible
    /// for.
    fn fallback_language(&self) -> &unic_langid::LanguageIdentifier {
        &self.fallback_language
    }
    /// The domain for the translation that this loader is associated with.
    fn domain(&self) -> &str {
        &self.domain
    }

    /// The language file name to use for this loader.
    fn language_file_name(&self) -> String {
        format!("{}.ftl", self.domain())
    }

    /// Get the language which is currently loaded for this loader.
    fn current_language(&self) -> unic_langid::LanguageIdentifier {
        self.locale_config.read().current_language.clone()
    }

    /// Load the languages `language_ids` using the resources packaged
    /// in the `i18n_assets` in order of fallback preference. This
    /// also sets the [LanguageLoader::current_language()] to the
    /// first in the `language_ids` slice. You can use
    /// [select()](super::select()) to determine which fallbacks are
    /// actually available for an arbitrary slice of preferences.
    fn load_languages(
        &self,
        i18n_assets: &dyn I18nAssets,
        language_ids: &[&unic_langid::LanguageIdentifier],
    ) -> Result<(), I18nEmbedError> {
        let current_language = *language_ids
            .get(0)
            .ok_or(I18nEmbedError::RequestedLanguagesEmpty)?;

        // The languages to load
        let mut load_language_ids = language_ids.to_vec();

        if !load_language_ids.contains(&&self.fallback_language) {
            load_language_ids.push(&self.fallback_language);
        }

        let mut locale_bundles = Vec::with_capacity(language_ids.len());

        for locale in load_language_ids {
            let (path, file) = self.language_file(&locale, i18n_assets);

            if let Some(file) = file {
                log::debug!(target:"i18n_embed::fluent", "Loaded language file: \"{0}\" for language: \"{1}\"", path, locale);

                let file_string = String::from_utf8(file.to_vec())
                    .map_err(|err| I18nEmbedError::ErrorParsingFileUtf8(path.clone(), err))?;

                let resource = match FluentResource::try_new(file_string) {
                    Ok(resource) => resource,
                    Err((resource, errors)) => {
                        errors.iter().for_each(|err| {
                            log::error!(target: "i18n_embed::fluent", "Error while parsing fluent language file \"{0}\": \"{1:?}\".", path, err);
                        });
                        resource
                    }
                };

                let mut resources = Vec::new();
                resources.push(Arc::new(resource));
                let locale_bundle = LocaleBundle::new(locale.clone(), resources);

                locale_bundles.push(locale_bundle);
            } else {
                log::debug!(target:"i18n_embed::fluent", "Unable to find language file: \"{0}\" for language: \"{1}\"", path, locale);
                if locale == &self.fallback_language {
                    return Err(I18nEmbedError::LanguageNotAvailable(path, locale.clone()));
                }
            }
        }

        let mut config_lock = self.locale_config.write();
        config_lock.locale_bundles = locale_bundles;
        config_lock.current_language = current_language.clone();
        drop(config_lock);

        Ok(())
    }
}
