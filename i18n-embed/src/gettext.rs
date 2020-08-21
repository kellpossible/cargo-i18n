use crate::{domain_from_module, I18nEmbedDyn, I18nEmbedError, LanguageLoader};
use tr;
use unic_langid::LanguageIdentifier;
use parking_lot::RwLock;

pub use gettext;

pub struct GettextLanguageLoader {
    current_language: RwLock<LanguageIdentifier>,
    module: &'static str,
    fallback_locale: LanguageIdentifier,
}

impl GettextLanguageLoader {
    pub fn new(module: &'static str, fallback_locale: unic_langid::LanguageIdentifier) -> Self {
        Self {
            current_language: RwLock::new(fallback_locale.clone()),
            module,
            fallback_locale,
        }
    }

    fn load_src_locale(&self) {
        let catalog = gettext::Catalog::empty();
        tr::set_translator!(catalog);
        *(self.current_language.write()) = self.fallback_locale().clone();
    }
}

impl LanguageLoader for GettextLanguageLoader {
    /// The fallback locale for the module this loader is responsible
    /// for.
    fn fallback_locale(&self) -> &LanguageIdentifier {
        &self.fallback_locale
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
    /// sets the `current_language()` to the first in the
    /// `language_ids` slice.
    ///
    /// **Note:** Gettext doesn't support loading multiple languages
    /// as multiple fallbacks. We only load the first of the requested
    /// languages, and the fallback is the src language.
    fn load_languages(
        &self,
        language_ids: &[&unic_langid::LanguageIdentifier],
        i18n_embed: &dyn I18nEmbedDyn,
    ) -> Result<(), I18nEmbedError> {
        let language_id = *language_ids.get(0).ok_or(I18nEmbedError::RequestedLanguagesEmpty)?;

        if language_id == self.fallback_locale() {
            self.load_src_locale();
            return Ok(());
        }

        let language_id_string = language_id.to_string();

        let file_path = format!("{}/{}", language_id_string, self.language_file_name());

        log::debug!("Loading language file: {}", file_path);

        let f = i18n_embed
            .get_dyn(file_path.as_ref())
            .ok_or(I18nEmbedError::LanguageNotAvailable(language_id_string))?;

        let catalog = gettext::Catalog::parse(&*f).expect("could not parse the catalog");
        tr::internal::set_translator(self.module, catalog);
        *(self.current_language.write()) = language_id.clone();

        Ok(())
    }
}
