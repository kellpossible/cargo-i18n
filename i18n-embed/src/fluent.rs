use crate::{domain_from_module, LanguageLoader, I18nEmbedDyn, I18nEmbedError};
use fluent::FluentValue;
use std::collections::HashMap;
use unic_langid::LanguageIdentifier;
use parking_lot::RwLock;

lazy_static::lazy_static! {
    static ref CURRENT_LANGUAGE: RwLock<LanguageIdentifier> = {
        let language = LanguageIdentifier::default();
        RwLock::new(language)
    };
}
pub struct FluentLanguageLoader {
    current_language: RwLock<LanguageIdentifier>,
    module: &'static str,
    fallback_locale: unic_langid::LanguageIdentifier,
}

impl FluentLanguageLoader {
    pub fn new(domain: &'static str, fallback_locale: unic_langid::LanguageIdentifier) -> Self {
        Self {
            current_language: RwLock::new(fallback_locale.clone()),
            module: domain,
            fallback_locale,
        }
    }

    pub fn get_locale(
        locale: &LanguageIdentifier,
        key: &'static str,
        args: HashMap<String, FluentValue>,
    ) -> String {
        todo!()
    }

    pub fn get(&self, key: &'static str, args: HashMap<String, FluentValue>) -> String {
        Self::get_locale(&self.current_language(), key, args)
    }
}

impl LanguageLoader for FluentLanguageLoader {
    /// The fallback locale for the module this loader is responsible
    /// for.
    fn fallback_locale(&self) -> &unic_langid::LanguageIdentifier {
        &self.fallback_locale
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
        self.current_language.read().clone()
    }

    /// Load the languages `language_ids` using the resources packaged
    /// in the `i18n_embed` in order of fallback preference. This also
    /// sets the `current_language()` to the first in the
    /// `language_ids` slice.
    fn load_languages(
        &self,
        language_ids: &[&unic_langid::LanguageIdentifier],
        i18n_embed: &dyn I18nEmbedDyn,
    ) -> Result<(), I18nEmbedError> {
        let language_id = *language_ids.get(0).ok_or(I18nEmbedError::RequestedLanguagesEmpty)?;

        let language_id_string = language_id.to_string();

        let file_path = format!("{}/{}", language_id_string, self.language_file_name());

        log::debug!("Loading language file: {}", file_path);

        let f = i18n_embed
            .get_dyn(file_path.as_ref())
            .ok_or(I18nEmbedError::LanguageNotAvailable(language_id_string))?;

        todo!()
    }
}
