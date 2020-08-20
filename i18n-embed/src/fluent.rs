use unic_langid::LanguageIdentifier;
use std::{collections::HashMap, sync::RwLock};
use lazy_static::lazy_static;
use crate::LanguageLoader;
use fluent::FluentValue;

lazy_static! {
    static ref CURRENT_LOCALE: RwLock<LanguageIdentifier> = {
        let locale: LanguageIdentifier = LanguageIdentifier::default();
        RwLock::new(locale)
    };
}

pub trait FluentLoader: LanguageLoader {
    fn get_locale(locale: &LanguageIdentifier, key: &'static str, args: HashMap<String, FluentValue>) -> String;
    fn get(&self, key: &'static str, args: HashMap<String, FluentValue>) -> String {
        Self::get_locale(&self.current_language(), key, args)
    }
}

pub struct Fluent {
    current_language: RwLock<LanguageIdentifier>,
}

pub struct FluentString {
    pub key: &'static str,
    
}