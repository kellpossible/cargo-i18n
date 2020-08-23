use crate::{domain_from_module, I18nEmbedDyn, I18nEmbedError, LanguageLoader};

pub use i18n_embed_impl::gettext_language_loader;

use parking_lot::RwLock;
use unic_langid::LanguageIdentifier;

pub struct GettextLanguageLoader {
    current_language: RwLock<LanguageIdentifier>,
    module: &'static str,
    fallback_language: LanguageIdentifier,
}

impl GettextLanguageLoader {
    pub fn new(module: &'static str, fallback_language: unic_langid::LanguageIdentifier) -> Self {
        Self {
            current_language: RwLock::new(fallback_language.clone()),
            module,
            fallback_language,
        }
    }

    fn load_src_language(&self) {
        let catalog = gettext_system::Catalog::empty();
        tr::internal::set_translator(self.module, catalog);
        *(self.current_language.write()) = self.fallback_language().clone();
    }
}

impl LanguageLoader for GettextLanguageLoader {
    /// The fallback language for the module this loader is responsible
    /// for.
    fn fallback_language(&self) -> &LanguageIdentifier {
        &self.fallback_language
    }

    /// The domain for the translation that this loader is associated with.
    fn domain(&self) -> &'static str {
        domain_from_module(self.module)
    }

    /// The language file name to use for this loader's domain.
    fn language_file_name(&self) -> String {
        format!("{}.mo", self.domain())
    }

    /// Get the language which is currently loaded for this loader.
    fn current_language(&self) -> LanguageIdentifier {
        self.current_language.read().clone()
    }

    /// Load the languages `language_ids` using the resources packaged
    /// in the `i18n_embed` in order of fallback preference. This also
    /// sets the [LanguageLoader::current_language()] to the first in
    /// the `language_ids` slice. You can use [select()](super::select())
    /// to determine which fallbacks are actually available for an
    /// arbitrary slice of preferences.
    ///
    /// **Note:** Gettext doesn't support loading multiple languages
    /// as multiple fallbacks. We only load the first of the requested
    /// languages, and the fallback is the src language.
    fn load_languages(
        &self,
        language_ids: &[&unic_langid::LanguageIdentifier],
        i18n_embed: &dyn I18nEmbedDyn,
    ) -> Result<(), I18nEmbedError> {
        let language_id = *language_ids
            .get(0)
            .ok_or(I18nEmbedError::RequestedLanguagesEmpty)?;

        if language_id == self.fallback_language() {
            self.load_src_language();
            return Ok(());
        }

        let (_path, file) = match self.language_file(&language_id, i18n_embed) {
            (path, Some(f)) => (path, f),
            (path, None) => {
                log::error!(
                    target:"i18n_embed::gettext", 
                    "{} Setting current_language to fallback locale: \"{}\".", 
                    I18nEmbedError::LanguageNotAvailable(path, language_id.clone()),
                    self.fallback_language);
                self.load_src_language();
                return Ok(());
            }
        };

        let catalog = gettext_system::Catalog::parse(&*file).expect("could not parse the catalog");
        tr::internal::set_translator(self.module, catalog);
        *(self.current_language.write()) = language_id.clone();

        Ok(())
    }
}
