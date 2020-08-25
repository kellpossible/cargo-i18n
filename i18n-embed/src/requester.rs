use crate::{I18nEmbedError, Localizer};
use std::{collections::HashMap, rc::Weak};

/// A trait used by [I18nEmbed](I18nEmbed) to ascertain which
/// languages are being requested.
pub trait LanguageRequester<'a> {
    /// Add a listener to this `LanguageRequester`. When the system
    /// reports that the currently requested languages has changed,
    /// each listener will have its
    /// [Localizer#select()](Localizer#select()) method called.
    ///
    /// If you haven't already selected a language for the localizer
    /// you are adding here, you may want to manually call
    /// [#poll()](#poll()) after adding the listener/s.
    fn add_listener(&mut self, localizer: Weak<dyn Localizer<'a>>);
    /// Poll the system's currently selected language, and call
    /// [Localizer#select()](Localizer#select()) on each of the
    /// listeners.
    fn poll(&mut self) -> Result<(), I18nEmbedError>;
    /// Override the languages fed to the [Localizer](Localizer) listeners during
    /// a [#poll()](#poll()). Set this as `None` to disable the override.
    fn set_language_override(
        &mut self,
        language_override: Option<unic_langid::LanguageIdentifier>,
    ) -> Result<(), I18nEmbedError>;
    /// The currently requested languages.
    fn requested_languages(&self) -> Vec<unic_langid::LanguageIdentifier>;
    /// The languages reported to be available in the
    /// listener [Localizer](Localizer)s.
    fn available_languages(&self) -> Result<Vec<unic_langid::LanguageIdentifier>, I18nEmbedError>;
    /// The languages currently loaded, keyed by the
    /// [LanguageLoader::domain()](crate::LanguageLoader::domain()).
    fn current_languages(&self) -> HashMap<&str, unic_langid::LanguageIdentifier>;
}

/// Provide the functionality for overrides and listeners for a
/// [LanguageRequester](LanguageRequester) implementation.
#[cfg(any(feature = "desktop-requester", feature = "web-sys-requester"))]
struct LanguageRequesterImpl<'a> {
    listeners: Vec<Weak<dyn Localizer<'a>>>,
    language_override: Option<unic_langid::LanguageIdentifier>,
}

#[cfg(any(feature = "desktop-requester", feature = "web-sys-requester"))]
impl<'a> LanguageRequesterImpl<'a> {
    /// Create a new [LanguageRequesterImpl](LanguageRequesterImpl).
    pub fn new() -> LanguageRequesterImpl<'a> {
        LanguageRequesterImpl {
            listeners: Vec::new(),
            language_override: None,
        }
    }

    /// Set an override for the requested language which is used when the
    /// [LanguageRequesterImpl#poll()](LanguageRequester#poll()) method
    /// is called. If `None`, then no override is used.
    fn set_language_override(
        &mut self,
        language_override: Option<unic_langid::LanguageIdentifier>,
    ) -> Result<(), I18nEmbedError> {
        self.language_override = language_override;
        Ok(())
    }

    fn add_listener(&mut self, localizer: Weak<dyn Localizer<'a>>) {
        self.listeners.push(localizer);
    }

    /// With the provided `requested_languages` call
    /// [Localizer#select()](Localizer#select()) on each of the
    /// listeners.
    fn poll_without_override(
        &mut self,
        requested_languages: Vec<unic_langid::LanguageIdentifier>,
    ) -> Result<(), I18nEmbedError> {
        let mut errors: Vec<I18nEmbedError> = Vec::new();

        self.listeners.retain(|listener| match listener.upgrade() {
            Some(l) => {
                match l.select(&requested_languages) {
                    Ok(_) => {}
                    Err(err) => {
                        errors.push(err);
                    }
                }

                true
            }
            None => false,
        });

        if errors.is_empty() {
            Ok(())
        } else if errors.len() == 1 {
            Err(errors.into_iter().next().unwrap())
        } else {
            Err(I18nEmbedError::Multiple(errors))
        }
    }

    /// With the provided `requested_languages` call
    /// [Localizer#select()](Localizer#select()) on each of the
    /// listeners. The `requested_languages` may be ignored if
    /// [#set_language_override()](#set_language_override()) has been
    /// set.
    pub fn poll(
        &mut self,
        requested_languages: Vec<unic_langid::LanguageIdentifier>,
    ) -> Result<(), I18nEmbedError> {
        let languages = match &self.language_override {
            Some(language) => {
                log::debug!("Using language override: {}", language);
                vec![language.clone()]
            }
            None => requested_languages,
        };

        self.poll_without_override(languages)
    }

    /// The languages reported to be available in the
    /// listener [Localizer](Localizer)s.
    fn available_languages(&self) -> Result<Vec<unic_langid::LanguageIdentifier>, I18nEmbedError> {
        let mut available_languages = std::collections::HashSet::new();
        for weak_listener in &self.listeners {
            if let Some(localizer) = weak_listener.upgrade() {
                localizer
                    .available_languages()?
                    .iter()
                    .for_each(|language| {
                        available_languages.insert(language.clone());
                    })
            }
        }

        Ok(available_languages.into_iter().collect())
    }

    fn current_languages(&self) -> HashMap<&str, unic_langid::LanguageIdentifier> {
        let mut current_languages = HashMap::new();
        for weak_listener in &self.listeners {
            if let Some(localizer) = weak_listener.upgrade() {
                let loader = localizer.language_loader();
                current_languages.insert(loader.domain(), loader.current_language());
            }
        }

        current_languages
    }
}

/// A [LanguageRequester](LanguageRequester) for the desktop platform,
/// supporting windows, linux and mac. It uses
/// [locale_config](locale_config) to select the language based on the
/// system selected language.
///
/// ⚠️ *This API requires the following crate features to be activated: `desktop-requester`.*
#[cfg(feature = "desktop-requester")]
pub struct DesktopLanguageRequester<'a> {
    implementation: LanguageRequesterImpl<'a>,
}

#[cfg(feature = "desktop-requester")]
impl<'a> LanguageRequester<'a> for DesktopLanguageRequester<'a> {
    fn requested_languages(&self) -> Vec<unic_langid::LanguageIdentifier> {
        Self::requested_languages()
    }

    fn add_listener(&mut self, localizer: Weak<dyn Localizer<'a>>) {
        self.implementation.add_listener(localizer)
    }

    fn set_language_override(
        &mut self,
        language_override: Option<unic_langid::LanguageIdentifier>,
    ) -> Result<(), I18nEmbedError> {
        self.implementation.set_language_override(language_override)
    }

    fn poll(&mut self) -> Result<(), I18nEmbedError> {
        self.implementation.poll(self.requested_languages())
    }

    fn available_languages(&self) -> Result<Vec<unic_langid::LanguageIdentifier>, I18nEmbedError> {
        self.implementation.available_languages()
    }

    fn current_languages(&self) -> HashMap<&str, unic_langid::LanguageIdentifier> {
        self.implementation.current_languages()
    }
}

#[cfg(feature = "desktop-requester")]
impl<'a> Default for DesktopLanguageRequester<'a> {
    fn default() -> Self {
        DesktopLanguageRequester::new()
    }
}

#[cfg(feature = "desktop-requester")]
impl<'a> DesktopLanguageRequester<'a> {
    pub fn new() -> DesktopLanguageRequester<'a> {
        DesktopLanguageRequester {
            implementation: LanguageRequesterImpl::new(),
        }
    }

    pub fn requested_languages() -> Vec<unic_langid::LanguageIdentifier> {
        use locale_config::{LanguageRange, Locale};

        let current_locale = Locale::current();

        let ids: Vec<unic_langid::LanguageIdentifier> = current_locale
            .tags_for("messages")
            .filter_map(|tag: LanguageRange| match tag.to_string().parse() {
                Ok(tag) => Some(tag),
                Err(err) => {
                    log::error!("Unable to parse your locale: {:?}", err);
                    None
                }
            })
            .collect();

        log::info!("Current Locale: {:?}", ids);

        ids
    }
}

/// A [LanguageRequester](LanguageRequester) for the `web-sys` web platform.
///
/// ⚠️ *This API requires the following crate features to be activated: `web-sys-requester`.*
#[cfg(feature = "web-sys-requester")]
pub struct WebLanguageRequester<'a> {
    implementation: LanguageRequesterImpl<'a>,
}

#[cfg(feature = "web-sys-requester")]
impl<'a> WebLanguageRequester<'a> {
    pub fn new() -> WebLanguageRequester<'a> {
        WebLanguageRequester {
            implementation: LanguageRequesterImpl::new(),
        }
    }

    pub fn requested_languages() -> Vec<unic_langid::LanguageIdentifier> {
        use fluent_langneg::convert_vec_str_to_langids_lossy;
        let window = web_sys::window().expect("no global `window` exists");
        let navigator = window.navigator();
        let languages = navigator.languages();

        let requested_languages =
            convert_vec_str_to_langids_lossy(languages.iter().map(|js_value| {
                js_value
                    .as_string()
                    .expect("language value should be a string.")
            }));

        requested_languages
    }
}

#[cfg(feature = "web-sys-requester")]
impl<'a> LanguageRequester<'a> for WebLanguageRequester<'a> {
    fn requested_languages(&self) -> Vec<unic_langid::LanguageIdentifier> {
        Self::requested_languages()
    }

    fn add_listener(&mut self, localizer: Weak<dyn Localizer<'a>>) {
        self.implementation.add_listener(localizer)
    }

    fn poll(&mut self) -> Result<(), I18nEmbedError> {
        self.implementation.poll(self.requested_languages())
    }

    fn set_language_override(
        &mut self,
        language_override: Option<unic_langid::LanguageIdentifier>,
    ) -> Result<(), I18nEmbedError> {
        self.implementation.set_language_override(language_override)
    }

    fn available_languages(&self) -> Result<Vec<unic_langid::LanguageIdentifier>, I18nEmbedError> {
        self.implementation.available_languages()
    }

    fn current_languages(&self) -> HashMap<&str, unic_langid::LanguageIdentifier> {
        self.implementation.current_languages()
    }
}
