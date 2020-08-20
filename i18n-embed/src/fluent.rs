use unic_langid::LanguageIdentifier;
use crate::LanguageLoader;
use fluent::FluentValue;
use std::{sync::RwLock, collections::HashMap};

lazy_static::lazy_static! {
    static ref CURRENT_LANGUAGE: RwLock<LanguageIdentifier> = {
        let language = LanguageIdentifier::default();
        RwLock::new(language)
    };
}


pub trait FluentLanguageLoader: LanguageLoader {
    fn get_locale(locale: &LanguageIdentifier, key: &'static str, args: HashMap<String, FluentValue>) -> String {
        todo!()
    }
    fn get(&self, key: &'static str, args: HashMap<String, FluentValue>) -> String {
        Self::get_locale(&self.current_language(), key, args)
    }
}

pub struct MyLanguageLoader {
    current_language: RwLock<LanguageIdentifier>,
}

impl LanguageLoader for MyLanguageLoader {
    /// The fallback locale for the module this loader is responsible
    /// for.
    fn fallback_locale(&self) -> unic_langid::LanguageIdentifier {
        "en-US".parse().unwrap()
    }
    /// The domain for the translation that this loader is associated with.
    fn domain(&self) -> &'static str {
        "test"
    }
    /// Load the language associated with [fallback_locale()](LanguageLoader#fallback_locale()).
    fn load_fallback_locale(&self) {
        *(self.current_language.write().expect("Unable to write to current_language")) = self.fallback_locale();
    }
    /// The language file name to use for this loader.
    fn language_file_name(&self) -> String {
        todo!();
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

pub struct FluentString {
    pub key: &'static str,
    
}