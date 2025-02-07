//! This module contains the types and functions to interact with the
//! `fluent` localization system.
//!
//! Most important is the [FluentLanguageLoader].
//!
//! ⚠️ *This module requires the following crate features to be activated: `fluent-system`.*

use crate::{I18nAssets, I18nEmbedError, LanguageLoader};

use arc_swap::ArcSwap;
pub use fluent_langneg::NegotiationStrategy;
pub use i18n_embed_impl::fluent_language_loader;

use fluent::{
    bundle::FluentBundle, FluentArgs, FluentAttribute, FluentMessage, FluentResource, FluentValue,
};
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
    /// Storage for language localization resources. Outer `Vec` is per language (as specified in
    /// [`LanguageConfig::language_map`]), inner Vec is for storage of multiple bundles per
    /// language, in order of priority (highest to lowest).
    language_bundles: Vec<Vec<LanguageBundle>>,
    /// This maps a `LanguageIdentifier` to the index inside the
    /// `language_bundles` vector.
    language_map: HashMap<LanguageIdentifier, usize>,
}

#[derive(Debug)]
struct CurrentLanguages {
    /// Languages currently selected.
    languages: Vec<LanguageIdentifier>,
    /// Indexes into the [`LanguageConfig::language_bundles`] associated the
    /// currently selected [`CurrentLanguages::languages`].
    indices: Vec<usize>,
}

#[derive(Debug)]
struct FluentLanguageLoaderInner {
    language_config: Arc<RwLock<LanguageConfig>>,
    current_languages: CurrentLanguages,
}

/// [LanguageLoader] implementation for the `fluent` localization
/// system. Also provides methods to access localizations which have
/// been loaded.
///
/// ⚠️ *This API requires the following crate features to be activated: `fluent-system`.*
#[derive(Debug)]
pub struct FluentLanguageLoader {
    inner: ArcSwap<FluentLanguageLoaderInner>,
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
            inner: ArcSwap::new(Arc::new(FluentLanguageLoaderInner {
                language_config: Arc::new(RwLock::new(config)),
                current_languages: CurrentLanguages {
                    languages: vec![fallback_language.clone()],
                    indices: vec![],
                },
            })),
            domain: domain.into(),
            fallback_language,
        }
    }

    fn current_language_impl(
        &self,
        inner: &FluentLanguageLoaderInner,
    ) -> unic_langid::LanguageIdentifier {
        inner
            .current_languages
            .languages
            .first()
            .map_or_else(|| self.fallback_language.clone(), Clone::clone)
    }

    /// The languages associated with each actual currently loaded language bundle.
    pub fn current_languages(&self) -> Vec<unic_langid::LanguageIdentifier> {
        self.inner.load().current_languages.languages.clone()
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
        let inner = self.inner.load();
        let language_config = inner.language_config.read();
        inner
            .current_languages
            .indices
            .iter()
            .map(|&idx| &language_config.language_bundles[idx])
            .flat_map(|language_bundles| language_bundles.iter())
            .find_map(|language_bundle| language_bundle
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
                            inner.current_languages.languages.first().unwrap_or(&self.fallback_language), message_id, errors
                        )
                    }
                    value.into()
                })
            )
            .unwrap_or_else(|| {
                log::error!(
                    target:"i18n_embed::fluent",
                    "Unable to find localization for language \"{}\" and id \"{}\".",
                    inner.current_languages.languages.first().unwrap_or(&self.fallback_language),
                    message_id
                );
                format!("No localization for id: \"{}\"", message_id)
            })
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

    /// Get a localized attribute referenced by the `message_id` and `attribute_id`.
    pub fn get_attr(&self, message_id: &str, attribute_id: &str) -> String {
        self.get_attr_args_fluent(message_id, attribute_id, None)
    }

    /// A non-generic version of [FluentLanguageLoader::get_attr_args()].
    pub fn get_attr_args_concrete<'args>(
        &self,
        message_id: &str,
        attribute_id: &str,
        args: HashMap<&'args str, FluentValue<'args>>,
    ) -> String {
        self.get_attr_args_fluent(
            message_id,
            attribute_id,
            hash_map_to_fluent_args(args).as_ref(),
        )
    }

    /// A non-generic version of [FluentLanguageLoader::get_attr_args()]
    /// accepting [FluentArgs] instead of a [HashMap].
    pub fn get_attr_args_fluent<'args>(
        &self,
        message_id: &str,
        attribute_id: &str,
        args: Option<&'args FluentArgs<'args>>,
    ) -> String {
        let inner = self.inner.load();
        let language_config = inner.language_config.read();
        let current_language = self.current_language_impl(&inner);

        language_config.language_bundles.iter()
            .flat_map(|language_bundles| language_bundles.iter())
            .find_map(|language_bundle| {
            language_bundle
                .bundle
                .get_message(message_id)
                .and_then(|m: FluentMessage<'_>| {
                    m.get_attribute(attribute_id)
                    .map(|a: FluentAttribute<'_>| {
                        a.value()
                    })
                })
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
        .unwrap_or_else(|| {
            log::error!(
                target:"i18n_embed::fluent",
                "Unable to find localization for language \"{}\", message id \"{}\" and attribute id \"{}\".",
                current_language,
                message_id,
                attribute_id
            );
            format!("No localization for message id: \"{message_id}\" and attribute id: \"{attribute_id}\"")
        })
    }

    /// Get a localized attribute referenced by the `message_id` and `attribute_id`, and
    /// formatted with the specified `args`.
    pub fn get_attr_args<'a, S, V>(
        &self,
        message_id: &str,
        attribute_id: &str,
        args: HashMap<S, V>,
    ) -> String
    where
        S: Into<Cow<'a, str>> + Clone,
        V: Into<FluentValue<'a>> + Clone,
    {
        self.get_attr_args_fluent(
            message_id,
            attribute_id,
            hash_map_to_fluent_args(args).as_ref(),
        )
    }

    /// available in any of the languages currently loaded (including
    /// the fallback language).
    pub fn has(&self, message_id: &str) -> bool {
        self.inner
            .load()
            .language_config
            .read()
            .language_bundles
            .iter()
            .flat_map(|language_bundles| language_bundles.iter())
            .any(|language_bundle| language_bundle.bundle.has_message(message_id))
    }

    /// Determines if an attribute associated with the specified `message_id`
    /// is available in any of the currently loaded languages, including the fallback language.
    ///
    /// Returns true if at least one available instance was found,
    /// false otherwise.
    ///
    /// Note that this also returns false if the `message_id` could not be found;
    /// use [FluentLanguageLoader::has()] to determine if the `message_id` is available.
    pub fn has_attr(&self, message_id: &str, attribute_id: &str) -> bool {
        self.inner
            .load()
            .language_config
            .read()
            .language_bundles
            .iter()
            .flat_map(|bundles| bundles.iter())
            .find_map(|bundle| {
                bundle
                    .bundle
                    .get_message(message_id)
                    .map(|message| message.get_attribute(attribute_id).is_some())
            })
            .unwrap_or(false)
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
        self.inner
            .load()
            .language_config
            .read()
            .language_bundles
            .iter()
            .flat_map(|language_bundles| language_bundles.iter())
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
        let inner = self.inner.load();
        let config_lock = inner.language_config.read();

        let mut iter = config_lock
            .language_bundles
            .iter()
            .flat_map(|language_bundles| language_bundles.iter())
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
        for bundle in self
            .inner
            .load()
            .language_config
            .write()
            .language_bundles
            .iter_mut()
            .flat_map(|bundles| bundles.iter_mut())
        {
            f(&mut bundle.bundle);
        }
    }

    /// Create a new loader with a subset of currently loaded languages.
    /// This is a rather cheap operation and does not require any
    /// extensive copy operations. Cheap does not mean free so you
    /// should not call this message repeatedly in order to translate
    /// multiple strings for the same language.
    pub fn select_languages<LI: AsRef<LanguageIdentifier>>(
        &self,
        languages: &[LI],
    ) -> FluentLanguageLoader {
        let inner = self.inner.load();
        let config_lock = inner.language_config.read();
        let fallback_language: Option<&unic_langid::LanguageIdentifier> = if languages
            .iter()
            .any(|language| language.as_ref() == &self.fallback_language)
        {
            None
        } else {
            Some(&self.fallback_language)
        };

        let indices = languages
            .iter()
            .map(|lang| lang.as_ref())
            .chain(fallback_language)
            .filter_map(|lang| config_lock.language_map.get(lang.as_ref()))
            .cloned()
            .collect();
        FluentLanguageLoader {
            inner: ArcSwap::new(Arc::new(FluentLanguageLoaderInner {
                current_languages: CurrentLanguages {
                    languages: languages.iter().map(|lang| lang.as_ref().clone()).collect(),
                    indices,
                },
                language_config: self.inner.load().language_config.clone(),
            })),
            domain: self.domain.clone(),
            fallback_language: self.fallback_language.clone(),
        }
    }

    /// Select the requested `languages` from the currently loaded languages using the supplied
    /// [`NegotiationStrategy`].
    pub fn select_languages_negotiate<LI: AsRef<LanguageIdentifier>>(
        &self,
        languages: &[LI],
        strategy: NegotiationStrategy,
    ) -> FluentLanguageLoader {
        let available_languages = &self.inner.load().current_languages.languages;
        let negotiated_languages = fluent_langneg::negotiate_languages(
            languages,
            available_languages,
            Some(self.fallback_language()),
            strategy,
        );

        self.select_languages(&negotiated_languages)
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

    /// Get the language which is currently selected for this loader.
    fn current_language(&self) -> unic_langid::LanguageIdentifier {
        self.current_language_impl(&self.inner.load())
    }

    /// Load the languages `language_ids` using the resources packaged
    /// in the `i18n_assets` in order of fallback preference. This
    /// also sets the [LanguageLoader::current_language()] to the
    /// first in the `language_ids` slice. You can use
    /// [select()](super::select()) to determine which fallbacks are
    /// actually available for an arbitrary slice of preferences.
    #[allow(single_use_lifetimes)]
    fn load_languages<'a>(
        &self,
        i18n_assets: &dyn I18nAssets,
        language_ids: &[unic_langid::LanguageIdentifier],
    ) -> Result<(), I18nEmbedError> {
        let mut language_ids = language_ids.iter().peekable();
        if language_ids.peek().is_none() {
            return Err(I18nEmbedError::RequestedLanguagesEmpty);
        }

        // The languages to load
        let language_ids: Vec<unic_langid::LanguageIdentifier> =
            language_ids.map(|id| (*id).clone()).collect();
        let mut load_language_ids: Vec<unic_langid::LanguageIdentifier> = language_ids.clone();

        if !load_language_ids.contains(&self.fallback_language) {
            load_language_ids.push(self.fallback_language.clone());
        }
        let language_bundles: Vec<Vec<_>> = load_language_ids.iter().map(|language| {
            let (path, files) = self.language_files(language, i18n_assets);

            if files.is_empty() {
                log::debug!(target:"i18n_embed::fluent", "Unable to find language file: \"{0}\" for language: \"{1}\"", path, language);
                if language == &self.fallback_language {
                    return Err(I18nEmbedError::LanguageNotAvailable(path, language.clone()));
                }
            }
            files.into_iter().map(|file| {
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

                Ok(LanguageBundle::new(language.clone(), resource))
            }).collect::<Result<Vec<_>, I18nEmbedError>>()
        }).collect::<Result<_, I18nEmbedError>>()?;

        self.inner.swap(Arc::new(FluentLanguageLoaderInner {
            current_languages: CurrentLanguages {
                languages: language_ids,
                indices: (0..load_language_ids.len()).collect(),
            },
            language_config: Arc::new(RwLock::new(LanguageConfig {
                language_map: language_bundles
                    .iter()
                    .enumerate()
                    .map(|(i, language_bundles)| {
                        (
                            language_bundles.first().expect("Expect there to be at least bundle in a set of bundles per language").language.clone(),
                            i
                        )
                    })
                    .collect(),
                language_bundles,
            })),
        }));

        Ok(())
    }

    fn reload(&self, i18n_assets: &dyn I18nAssets) -> Result<(), I18nEmbedError> {
        self.load_languages(
            i18n_assets,
            &self.inner.load().current_languages.languages.clone(),
        )
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
