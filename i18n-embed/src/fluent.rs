use crate::{domain_from_module, I18nEmbedDyn, I18nEmbedError, LanguageLoader};

pub use i18n_embed_impl::fluent_language_loader;

use fluent::{concurrent::FluentBundle, FluentMessage, FluentResource, FluentValue};
use fluent_syntax::ast::Pattern;
use parking_lot::RwLock;
use std::{borrow::Cow, collections::HashMap, sync::Arc};
use unic_langid::LanguageIdentifier;

lazy_static::lazy_static! {
    static ref CURRENT_LANGUAGE: RwLock<LanguageIdentifier> = {
        let language = LanguageIdentifier::default();
        RwLock::new(language)
    };
}

struct LocaleBundle {
    pub locale: LanguageIdentifier,
    pub bundle: FluentBundle<Arc<FluentResource>>,
}

impl LocaleBundle {
    pub fn new(locale: LanguageIdentifier, resources: Vec<Arc<FluentResource>>) -> Self {
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

struct LocaleConfig {
    current_language: LanguageIdentifier,
    locale_bundles: Vec<LocaleBundle>,
}

pub struct FluentLanguageLoader {
    locale_config: RwLock<LocaleConfig>,
    module: &'static str,
    fallback_language: unic_langid::LanguageIdentifier,
}

impl FluentLanguageLoader {
    pub fn new(domain: &'static str, fallback_language: unic_langid::LanguageIdentifier) -> Self {
        let config = LocaleConfig {
            current_language: fallback_language.clone(),
            locale_bundles: Vec::new(),
        };

        Self {
            locale_config: RwLock::new(config),
            module: domain,
            fallback_language,
        }
    }

    pub fn get(&self, id: &'static str) -> String {
        self.get_args_concrete(id, HashMap::new())
    }

    /// A non-generic version of [FluentLanguageLoader::get_args()].
    pub fn get_args_concrete<'a>(
        &self,
        id: &'static str,
        args: HashMap<&'a str, FluentValue<'a>>,
    ) -> String {
        let config_lock = self.locale_config.read();

        let args = if args.is_empty() { None } else { Some(&args) };

        config_lock.locale_bundles.iter().filter_map(|locale_bundle| {
            locale_bundle
                .bundle
                .get_message(id)
                .and_then(|m: FluentMessage| m.value)
                .map(|pattern: &Pattern| {
                    let mut errors = Vec::new();
                    let value = locale_bundle.bundle.format_pattern(pattern, args, &mut errors);
                    if !errors.is_empty() {
                        log::error!(
                            target:"i18n_embed::fluent",
                            "Failed to format a message for locale \"{}\" and id \"{}\".\nErrors\n{:?}.",
                            &config_lock.current_language, id, errors
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
                    id
                );
                format!("No localization for id: \"{}\"", id)
            })
    }

    pub fn get_args<'a, S, V>(&self, id: &'static str, args: HashMap<S, V>) -> String
    where
        S: Into<Cow<'a, str>> + Clone + 'static,
        V: Into<FluentValue<'a>> + Clone + 'static,
    {
        let mut keys: Vec<Cow<'a, str>> = Vec::new();

        let mut map: HashMap<&str, FluentValue> = HashMap::with_capacity(args.len());

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
}

impl LanguageLoader for FluentLanguageLoader {
    /// The fallback language for the module this loader is responsible
    /// for.
    fn fallback_language(&self) -> &unic_langid::LanguageIdentifier {
        &self.fallback_language
    }
    /// The domain for the translation that this loader is associated with.
    fn domain(&self) -> &'static str {
        domain_from_module(self.module)
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
    /// in the `i18n_embed` in order of fallback preference. This also
    /// sets the [LanguageLoader::current_language()] to the first in
    /// the `language_ids` slice. You can use
    /// [select()](super::select()) to determine which fallbacks are
    /// actually available for an arbitrary slice of preferences.
    fn load_languages(
        &self,
        language_ids: &[&unic_langid::LanguageIdentifier],
        i18n_embed: &dyn I18nEmbedDyn,
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
            let (path, file) = self.language_file(&locale, i18n_embed);

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
                    return Err(I18nEmbedError::LanguageNotAvailable(
                        path.clone(),
                        locale.clone(),
                    ));
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
