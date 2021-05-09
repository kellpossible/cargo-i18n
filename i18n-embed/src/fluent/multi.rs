//! This module contains the [FluentMultiLanguageLoader].
//!
//! It acts as a map, that holds every translation for easy access.
//! Since this loads everything in memory, you should be aware that it may not be optimal for usage
//! on systems with limited memory.
//!
//! This module contains the types and functions to interact with the
//! `fluent` localization system.
//!
//! ⚠️ *This module requires the following crate features to be activated: `fluent-system`.*

use std::{borrow::Cow, collections::HashMap, fmt::Debug};

use fluent::{FluentArgs, FluentMessage, FluentValue};
use fluent_syntax::ast::{self, Pattern};
use parking_lot::RwLock;
use unic_langid::LanguageIdentifier;

use crate::fluent::{files_to_fluent_bundle, LanguageBundle};
use crate::{I18nAssets, I18nEmbedError, LanguageLoader};

#[derive(Debug)]
struct MultiLanguageConfig {
    /// A Hashmap which contains every available locale.
    language_bundles: HashMap<LanguageIdentifier, LanguageBundle>,
}

/// [LanguageLoader] implementation for the `fluent` localization
/// system. Also provides methods to access localizations which have
/// been loaded.
///
/// ⚠️ *This API requires the following crate features to be activated: `fluent-system`.*
#[derive(Debug)]
pub struct FluentMultiLanguageLoader {
    language_config: RwLock<MultiLanguageConfig>,
    domain: String,
    fallback_language: unic_langid::LanguageIdentifier,
}

impl FluentMultiLanguageLoader {
    /// Create a new `FluentMultiLanguageLoader`, which loads messages for
    /// the specified `domain`, and relies on the specified
    /// `fallback_language` for any messages that do not exist for the
    /// current language.
    pub fn new<S: Into<String>>(
        domain: S,
        fallback_language: unic_langid::LanguageIdentifier,
    ) -> Self {
        let config = MultiLanguageConfig {
            language_bundles: HashMap::new(),
        };

        Self {
            language_config: RwLock::new(config),
            domain: domain.into(),
            fallback_language,
        }
    }

    /// The languages associated with each actual loaded language bundle.
    pub fn current_languages(&self) -> Vec<unic_langid::LanguageIdentifier> {
        self.language_config
            .read()
            .language_bundles
            .iter()
            .map(|(_, b)| b.language.clone())
            .collect()
    }

    /// Returns translated string from specified locale.
    /// Automatically fallback to global language, you don't need to input it again.
    pub fn get_with_locale<'a>(
        &self,
        locale: &'a LanguageIdentifier,
        message_id: &'a str,
    ) -> String {
        self.get_args_concrete(&[locale], message_id, HashMap::new())
    }

    /// Returns translated string from specified locale.
    /// If it doesn't exists, will fallback to global locale.
    pub fn get_with_locale_and_args<'a, S, V>(
        &self,
        locale: &LanguageIdentifier,
        message_id: &str,
        args: HashMap<S, V>,
    ) -> String
    where
        S: Into<Cow<'a, str>> + Clone,
        V: Into<FluentValue<'a>> + Clone,
    {
        let args = crate::fluent::prepare_args_map(args);
        self.get_args_concrete(&[locale], message_id, args)
    }

    /// Fallback to custom specified fallback locales in order.
    /// Automatically fallback to global language, you don't need to input it again.
    pub fn get_with_custom_fallback<'source, I>(
        &self,
        fallback_locales: I,
        message_id: &'source str,
    ) -> String
    where
        I: AsRef<[&'source LanguageIdentifier]>,
    {
        self.get_args_concrete(fallback_locales.as_ref(), message_id, HashMap::new())
    }

    /// Returns translated string, accepting multiple fallback locale, ultimately falling back the global fallback
    /// language specified on [FluentMultiLanguageLoader].
    ///
    /// Useful for supporting multiple versions of the same base language, such as es-AR, es-ES.
    pub fn get_with_custom_fallback_and_args<'a, I, S, V>(
        &self,
        fallback_locales: I,
        message_id: &str,
        args: HashMap<S, V>,
    ) -> String
    where
        I: AsRef<[&'a LanguageIdentifier]>,
        S: Into<Cow<'a, str>> + Clone,
        V: Into<FluentValue<'a>> + Clone,
    {
        let args = crate::fluent::prepare_args_map(args);
        self.get_args_concrete(fallback_locales, message_id, args)
    }

    /// A non-generic version of [FluentLanguageLoader::get_args()].
    pub fn get_args_concrete<'source, I>(
        &self,
        locales: I,
        message_id: &str,
        args: HashMap<Cow<'source, str>, FluentValue<'source>>,
    ) -> String
    where
        I: AsRef<[&'source LanguageIdentifier]>,
    {
        let args_option = if args.is_empty() {
            None
        } else {
            let mut fluent_args = FluentArgs::with_capacity(args.len());

            for (key, value) in args {
                fluent_args.set(key, value);
            }

            Some(fluent_args)
        };

        self.get_args_fluent(message_id, locales.as_ref(), args_option.as_ref())
    }

    /// A non-generic version of [FluentLanguageLoader::get_args()]
    /// accepting [FluentArgs] instead of a [HashMap].
    pub fn get_args_fluent<'args>(
        &self,
        message_id: &str,
        locales_fallback: &[&LanguageIdentifier],
        args: Option<&'args FluentArgs<'args>>,
    ) -> String {
        let config_lock = self.language_config.read();

        let mut locales_fallback = locales_fallback.to_vec();
        locales_fallback.push(&self.fallback_language);

        locales_fallback.iter().find_map(|language_id| {
            // retrieves message with args if message-id exists inside this locale
            config_lock
                .language_bundles
                .get(language_id)
                .and_then(|language_bundle| {
                    language_bundle.bundle.get_message(message_id)
                    .and_then(|m: FluentMessage<'_>| m.value())
                        .map(|pattern: &Pattern<&str>| {
                            let mut errors = Vec::new();
                            let value = language_bundle.bundle.format_pattern(pattern, args, &mut errors);
                            if !errors.is_empty() {
                                log::error!(
                                    target: "i18n_embed::fluent",
                                    "Failed to format a message for language \"{}\" and id \"{}\".\nErrors\n{:?}.",
                                    &language_id, message_id, errors
                                )
                            }
                            value.to_string()
                        })
                })
        })
            .unwrap_or_else(|| {
                log::error!(
                    target: "i18n_embed::fluent",
                    "Unable to find localization for id \"{}\" on any language.",
                    message_id
                );
                format!("No localization for id: \"{}\"", message_id)
            })
    }

    /// Returns true if a message with the specified `message_id` is
    /// available in any of the languages currently loaded (including
    /// the fallback language).
    pub fn has(&self, message_id: &str) -> bool {
        let config_lock = self.language_config.read();
        let mut has_message = false;

        config_lock
            .language_bundles
            .iter()
            .for_each(|(_, language_bundle)| {
                has_message |= language_bundle.bundle.has_message(message_id)
            });

        has_message
    }

    /// Run the `closure` with the message that matches the specified
    /// `message_id` (if it is available in any of the languages
    /// currently loaded, including the fallback language). Returns
    /// `Some` of whatever whatever the closure returns, or `None` if
    /// no messages were found matching the `message_id`.
    pub fn with_fluent_message<OUT, C>(&self, message_id: &str, closure: C) -> Option<OUT>
    where
        C: Fn(fluent::FluentMessage<'_>) -> OUT,
    {
        let config_lock = self.language_config.read();

        config_lock
            .language_bundles
            .iter()
            .filter_map(|(_, language_bundle)| language_bundle.bundle.get_message(message_id))
            .next()
            .map(closure)
    }

    /// Runs the provided `closure` with an iterator over the messages
    /// available for the specified `language`. There may be duplicate
    /// messages when they are duplicated in resources applicable to
    /// the language. Returns the result of the closure.
    pub fn with_message_iter<OUT, C>(&self, language: &LanguageIdentifier, closure: C) -> OUT
    where
        C: Fn(&mut dyn Iterator<Item = &ast::Message<&str>>) -> OUT,
    {
        let config_lock = self.language_config.read();

        let mut iter = config_lock
            .language_bundles
            .iter()
            .filter(|(_, language_bundle)| &language_bundle.language == language)
            .flat_map(|(_, language_bundle)| {
                language_bundle.resources.iter().flat_map(|resource| {
                    resource.entries().filter_map(|entry| match entry {
                        ast::Entry::Message(message) => Some(message),
                        _ => None,
                    })
                })
            });

        (closure)(&mut iter)
    }

    /// Set whether the underlying Fluent logic should insert Unicode
    /// Directionality Isolation Marks around placeables.
    ///
    /// See [`fluent::bundle::FluentBundleBase::set_use_isolating`] for more
    /// information.
    ///
    /// **Note:** This function will have no effect if
    /// [`LanguageLoader::load_languages`] has not been called first.
    ///
    /// Default: `true`.
    pub fn set_use_isolating(&self, value: bool) {
        for bundle in self.language_config.write().language_bundles.values_mut() {
            bundle.bundle.set_use_isolating(value);
        }
    }
}

impl LanguageLoader for FluentMultiLanguageLoader {
    /// The fallback language for the module this loader is responsible for.
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

    fn current_language(&self) -> LanguageIdentifier {
        unimplemented!()
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
        // The languages to load
        let mut load_language_ids = language_ids.to_vec();

        if !load_language_ids.contains(&&self.fallback_language) {
            load_language_ids.push(&self.fallback_language);
        }

        let mut language_spec_bundles = HashMap::with_capacity(language_ids.len());

        for language in load_language_ids {
            let fluent_bundle =
                files_to_fluent_bundle(self, i18n_assets, language, &self.fallback_language)?;
            language_spec_bundles.insert(language.clone(), fluent_bundle);
        }

        let mut config_lock = self.language_config.write();
        config_lock.language_bundles = language_spec_bundles;
        drop(config_lock);

        Ok(())
    }
}
