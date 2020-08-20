use unic_langid::LanguageIdentifier;
use crate::{domain_from_module, LanguageLoader};
use fluent::FluentValue;
use std::{sync::RwLock, collections::HashMap};

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

    pub fn get_locale(locale: &LanguageIdentifier, key: &'static str, args: HashMap<String, FluentValue>) -> String {
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
    /// Load the language associated with [fallback_locale()](LanguageLoader#fallback_locale()).
    fn load_fallback_locale(&self) {
        *(self.current_language.write().expect("Unable to write to current_language")) = self.fallback_locale().clone();
    }
    /// The language file name to use for this loader.
    fn language_file_name(&self) -> String {
        format!("{}.ftl", self.domain())
    }
    /// Get the language which is currently loaded for this loader.
    fn current_language(&self) -> unic_langid::LanguageIdentifier {
        self.current_language.read().unwrap().clone()
    }

    fn load_language(
        &self,
        language_id: &LanguageIdentifier,
        i18n_embed: &dyn crate::I18nEmbedDyn,
    ) -> Result<(), crate::I18nEmbedError> {
        todo!()
    }
}