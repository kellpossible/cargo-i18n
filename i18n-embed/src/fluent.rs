//! This module contains the types and functions to interact with the
//! `fluent` localization system.
//!
//! Most important is the [FluentLanguageLoader].
//!
//! ⚠️ *This module requires the following crate features to be activated: `fluent-system`.*

use crate::{I18nAssets, I18nEmbedError, LanguageLoader};

pub use i18n_embed_impl::fluent_language_loader;

use fluent::{bundle::FluentBundle, FluentArgs, FluentMessage, FluentResource, FluentValue};
use fluent_syntax::ast::{self, Pattern};
use intl_memoizer::concurrent::IntlLangMemoizer;
use parking_lot::RwLock;
use std::{borrow::Cow, collections::HashMap, fmt::Debug, iter::FromIterator, sync::Arc};
use unic_langid::LanguageIdentifier;

struct LanguageBundle {
    language: LanguageIdentifier,
    bundle: FluentBundle<Arc<FluentResource>, IntlLangMemoizer>,
    resource: Arc<FluentResource>,
}

impl LanguageBundle {
    fn new(language: LanguageIdentifier, resource: FluentResource) -> Self {
        let mut bundle = FluentBundle::new_concurrent(vec![language.clone()]);
        let resource = Arc::new(resource);

        if let Err(errors) = bundle.add_resource(resource.clone()) {
            errors.iter().for_each(|error | {
                log::error!(target: "i18n_embed::fluent", "Error while adding resource to bundle: {0:?}.", error);
            })
        }

        Self {
            language,
            bundle,
            resource,
        }
    }
}

impl Debug for LanguageBundle {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "LanguageBundle(language: {})", self.language)
    }
}

#[derive(Debug)]
struct LanguageConfig {
    language_bundles: Vec<LanguageBundle>,
    /// This maps a `LanguageIdentifier` to the index inside the
    /// `language_bundles` vector.
    language_map: HashMap<LanguageIdentifier, usize>,
}

/// [LanguageLoader] implemenation for the `fluent` localization
/// system. Also provides methods to access localizations which have
/// been loaded.
///
/// ⚠️ *This API requires the following crate features to be activated: `fluent-system`.*
#[derive(Debug)]
pub struct FluentLanguageLoader {
    language_config: RwLock<LanguageConfig>,
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
        let config = LanguageConfig {
            language_bundles: Vec::new(),
            language_map: HashMap::new(),
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
            .map(|b| b.language.clone())
            .collect()
    }

    /// Get a localized message referenced by the `message_id`.
    pub fn get(&self, message_id: &str) -> String {
        self.get_args_fluent(message_id, None)
    }

    /// A non-generic version of [FluentLanguageLoader::get_args()].
    pub fn get_args_concrete<'args>(
        &self,
        message_id: &str,
        args: HashMap<&'args str, FluentValue<'args>>,
    ) -> String {
        self.get_args_fluent(message_id, hash_map_to_fluent_args(args).as_ref())
    }

    /// A non-generic version of [FluentLanguageLoader::get_args()]
    /// accepting [FluentArgs] instead of a [HashMap].
    pub fn get_args_fluent<'args>(
        &self,
        message_id: &str,
        args: Option<&'args FluentArgs<'args>>,
    ) -> String {
        let language_config = self.language_config.read();
        self._get(
            language_config.language_bundles.iter(),
            &self.current_language(),
            message_id,
            args,
        )
    }

    /// Get a localized message referenced by the `message_id`, and
    /// formatted with the specified `args`.
    pub fn get_args<'a, S, V>(&self, id: &str, args: HashMap<S, V>) -> String
    where
        S: Into<Cow<'a, str>> + Clone,
        V: Into<FluentValue<'a>> + Clone,
    {
        self.get_args_fluent(id, hash_map_to_fluent_args(args).as_ref())
    }

    /// Get a localized message referenced by the `message_id`.
    pub fn get_lang(&self, lang: &[&LanguageIdentifier], message_id: &str) -> String {
        self.get_lang_args_fluent(lang, message_id, None)
    }

    /// A non-generic version of [FluentLanguageLoader::get_lang_args()].
    pub fn get_lang_args_concrete<'source>(
        &self,
        lang: &[&LanguageIdentifier],
        message_id: &str,
        args: HashMap<&'source str, FluentValue<'source>>,
    ) -> String {
        self.get_lang_args_fluent(lang, message_id, hash_map_to_fluent_args(args).as_ref())
    }

    /// A non-generic version of [FluentLanguageLoader::get_lang_args()]
    /// accepting [FluentArgs] instead of a [HashMap].
    pub fn get_lang_args_fluent<'args>(
        &self,
        lang: &[&LanguageIdentifier],
        message_id: &str,
        args: Option<&'args FluentArgs<'args>>,
    ) -> String {
        let current_language = if lang.is_empty() {
            &self.fallback_language
        } else {
            lang[0]
        };
        let fallback_language = if lang.contains(&&self.fallback_language) {
            None
        } else {
            Some(&self.fallback_language)
        };
        let config_lock = self.language_config.read();
        let language_bundles = lang
            .iter()
            .chain(fallback_language.as_ref().into_iter())
            .filter_map(|id| {
                config_lock
                    .language_map
                    .get(id)
                    .map(|idx| &config_lock.language_bundles[*idx])
            });
        self._get(language_bundles, current_language, message_id, args)
    }

    /// Get a localized message for the given language identifiers, referenced
    /// by the `message_id` and formatted with the specified `args`.
    pub fn get_lang_args<'a, S, V>(
        &self,
        lang: &[&LanguageIdentifier],
        id: &str,
        args: HashMap<S, V>,
    ) -> String
    where
        S: Into<Cow<'a, str>> + Clone,
        V: Into<FluentValue<'a>> + Clone,
    {
        let fluent_args = hash_map_to_fluent_args(args);
        self.get_lang_args_fluent(lang, id, fluent_args.as_ref())
    }

    fn _get<'a, 'args>(
        &'a self,
        language_bundles: impl Iterator<Item = &'a LanguageBundle>,
        current_language: &LanguageIdentifier,
        message_id: &str,
        args: Option<&'args FluentArgs<'args>>,
    ) -> String {
        language_bundles.filter_map(|language_bundle| {
            language_bundle
                .bundle
                .get_message(message_id)
                .and_then(|m: FluentMessage<'_>| m.value())
                .map(|pattern: &Pattern<&str>| {
                    let mut errors = Vec::new();
                    let value = language_bundle.bundle.format_pattern(pattern, args, &mut errors);
                    if !errors.is_empty() {
                        log::error!(
                            target:"i18n_embed::fluent",
                            "Failed to format a message for language \"{}\" and id \"{}\".\nErrors\n{:?}.",
                            current_language, message_id, errors
                        )
                    }
                    value.into()
                })
            })
            .next()
            .unwrap_or_else(|| {
                log::error!(
                    target:"i18n_embed::fluent",
                    "Unable to find localization for language \"{}\" and id \"{}\".",
                    current_language,
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
            .for_each(|language_bundle| {
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
            .find_map(|language_bundle| language_bundle.bundle.get_message(message_id))
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
            .filter(|language_bundle| &language_bundle.language == language)
            .flat_map(|language_bundle| {
                language_bundle
                    .resource
                    .entries()
                    .filter_map(|entry| match entry {
                        ast::Entry::Message(message) => Some(message),
                        _ => None,
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
        self.with_bundles_mut(|bundle| bundle.set_use_isolating(value));
    }

    /// Apply some configuration to each budle in this loader.
    ///
    /// **Note:** This function will have no effect if
    /// [`LanguageLoader::load_languages`] has not been called first.
    pub fn with_bundles_mut<F>(&self, f: F)
    where
        F: Fn(&mut FluentBundle<Arc<FluentResource>, IntlLangMemoizer>),
    {
        for bundle in self.language_config.write().language_bundles.as_mut_slice() {
            f(&mut bundle.bundle);
        }
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
        let config_lock = self.language_config.read();
        config_lock.language_bundles.first().map_or_else(
            || self.fallback_language.clone(),
            |bundle| bundle.language.clone(),
        )
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
        language_ids
            .first()
            .ok_or(I18nEmbedError::RequestedLanguagesEmpty)?;

        // The languages to load
        let mut load_language_ids = language_ids.to_vec();

        if !load_language_ids.contains(&&self.fallback_language) {
            load_language_ids.push(&self.fallback_language);
        }

        let mut language_bundles = Vec::with_capacity(language_ids.len());

        for language in load_language_ids {
            let (path, file) = self.language_file(language, i18n_assets);

            if let Some(file) = file {
                log::debug!(target:"i18n_embed::fluent", "Loaded language file: \"{0}\" for language: \"{1}\"", path, language);

                let file_string = String::from_utf8(file.to_vec())
                    .map_err(|err| I18nEmbedError::ErrorParsingFileUtf8(path.clone(), err))?
                    // TODO: Workaround for https://github.com/kellpossible/cargo-i18n/issues/57
                    // remove when https://github.com/projectfluent/fluent-rs/issues/213 is resolved.
                    .replace("\u{000D}\n", "\n");

                let resource = match FluentResource::try_new(file_string) {
                    Ok(resource) => resource,
                    Err((resource, errors)) => {
                        errors.iter().for_each(|err| {
                            log::error!(target: "i18n_embed::fluent", "Error while parsing fluent language file \"{0}\": \"{1:?}\".", path, err);
                        });
                        resource
                    }
                };

                let language_bundle = LanguageBundle::new(language.clone(), resource);

                language_bundles.push(language_bundle);
            } else {
                log::debug!(target:"i18n_embed::fluent", "Unable to find language file: \"{0}\" for language: \"{1}\"", path, language);
                if language == &self.fallback_language {
                    return Err(I18nEmbedError::LanguageNotAvailable(path, language.clone()));
                }
            }
        }

        let mut config_lock = self.language_config.write();
        config_lock.language_bundles = language_bundles;
        config_lock.language_map = config_lock
            .language_bundles
            .iter()
            .enumerate()
            .map(|(i, language_bundle)| (language_bundle.language.clone(), i))
            .collect();
        drop(config_lock);

        Ok(())
    }
}

fn hash_map_to_fluent_args<'args, K, V>(map: HashMap<K, V>) -> Option<FluentArgs<'args>>
where
    K: Into<Cow<'args, str>>,
    V: Into<FluentValue<'args>>,
{
    if map.is_empty() {
        None
    } else {
        Some(FluentArgs::from_iter(map))
    }
}
