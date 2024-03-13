use crate::{I18nEmbedError, Localizer};
use std::{collections::HashMap, sync::Weak};

/// A trait used by [I18nAssets](crate::I18nAssets) to ascertain which
/// languages are being requested.
pub trait LanguageRequester<'a> {
    /// The [`Localizer`] being used with this [`LanguageRequester`].
    type Localizer: crate::Localizer;
    /// Add a listener to this `LanguageRequester`. When the system
    /// reports that the currently requested languages has changed,
    /// each listener will have its
    /// [Localizer#select()](Localizer#select()) method called. [Weak]
    /// is used so that when the [Arc](std::sync::Arc) that it references
    /// is dropped, the listener will also be removed next time this
    /// requester is polled/updates.
    ///
    /// If you haven't already selected a language for the localizer
    /// you are adding here, you may want to manually call
    /// [#poll()](#poll()) after adding the listener/s.
    fn add_listener(&mut self, listener: Weak<Self::Localizer>);
    /// Add a listener to this `LanguageRequester`. When the system
    /// reports that the currently requested languages has changed,
    /// each listener will have its
    /// [Localizer#select()](Localizer#select()) method called. As
    /// opposed to [LanguageRequester::add_listener()], this listener
    /// will not be removed.
    ///
    /// If you haven't already selected a language for the localizer
    /// you are adding here, you may want to manually call
    /// [#poll()](#poll()) after adding the listener/s.
    fn add_listener_ref(&mut self, listener: &'a Self::Localizer);
    /// Poll the system's currently selected language, and call
    /// [Localizer#select()](Localizer#select()) on each of the
    /// listeners.
    ///
    /// **NOTE:** Support for this across systems currently
    /// varies, it may not change when the system requested language
    /// changes during runtime without restarting your application. In
    /// the future some platforms may also gain support for automatic
    /// triggering when the requested display language changes.
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
    fn current_languages(&self) -> HashMap<String, unic_langid::LanguageIdentifier>;
}

/// Provide the functionality for overrides and listeners for a
/// [LanguageRequester](LanguageRequester) implementation.
pub struct LanguageRequesterImpl<'a, LOCALIZER> {
    arc_listeners: Vec<Weak<LOCALIZER>>,
    ref_listeners: Vec<&'a LOCALIZER>,
    language_override: Option<unic_langid::LanguageIdentifier>,
}

impl<'a, LOCALIZER> LanguageRequesterImpl<'a, LOCALIZER>
where
    LOCALIZER: Localizer,
{
    /// Create a new [LanguageRequesterImpl](LanguageRequesterImpl).
    pub fn new() -> LanguageRequesterImpl<'a, LOCALIZER> {
        LanguageRequesterImpl {
            arc_listeners: Vec::new(),
            ref_listeners: Vec::new(),
            language_override: None,
        }
    }

    /// Set an override for the requested language which is used when the
    /// [LanguageRequesterImpl#poll()](LanguageRequester#poll()) method
    /// is called. If `None`, then no override is used.
    pub fn set_language_override(
        &mut self,
        language_override: Option<unic_langid::LanguageIdentifier>,
    ) -> Result<(), I18nEmbedError> {
        self.language_override = language_override;
        Ok(())
    }

    /// Add a weak reference to a [Localizer], which listens to
    /// changes to the current language.
    pub fn add_listener(&mut self, listener: Weak<LOCALIZER>) {
        self.arc_listeners.push(listener);
    }

    /// Add a reference to [Localizer], which listens to changes to
    /// the current language.
    pub fn add_listener_ref(&mut self, listener: &'a LOCALIZER) {
        self.ref_listeners.push(listener);
    }

    /// With the provided `requested_languages` call
    /// [Localizer#select()](Localizer#select()) on each of the
    /// listeners.
    pub fn poll_without_override(
        &mut self,
        requested_languages: Vec<unic_langid::LanguageIdentifier>,
    ) -> Result<(), I18nEmbedError> {
        let mut errors: Vec<I18nEmbedError> = Vec::new();

        self.arc_listeners
            .retain(|listener| match listener.upgrade() {
                Some(arc_listener) => {
                    if let Err(error) = arc_listener.select(&requested_languages) {
                        errors.push(error);
                    }

                    true
                }
                None => false,
            });

        for boxed_listener in &self.ref_listeners {
            if let Err(error) = boxed_listener.select(&requested_languages) {
                errors.push(error);
            }
        }

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
    pub fn available_languages(
        &self,
    ) -> Result<Vec<unic_langid::LanguageIdentifier>, I18nEmbedError> {
        let mut available_languages = std::collections::HashSet::new();

        for weak_arc_listener in &self.arc_listeners {
            if let Some(arc_listener) = weak_arc_listener.upgrade() {
                arc_listener
                    .available_languages()?
                    .iter()
                    .for_each(|language| {
                        available_languages.insert(language.clone());
                    })
            }
        }

        for boxed_listener in &self.ref_listeners {
            boxed_listener
                .available_languages()?
                .iter()
                .for_each(|language| {
                    available_languages.insert(language.clone());
                })
        }

        Ok(available_languages.into_iter().collect())
    }

    /// Gets a `HashMap` with what each language is currently set
    /// (value) per domain (key).
    pub fn current_languages(&self) -> HashMap<String, unic_langid::LanguageIdentifier> {
        let mut current_languages = HashMap::new();
        for weak_listener in &self.arc_listeners {
            if let Some(localizer) = weak_listener.upgrade() {
                let loader = localizer.language_loader();
                current_languages.insert(loader.domain().to_string(), loader.current_language());
            }
        }

        current_languages
    }
}

impl<LOCALIZER: Localizer> Default for LanguageRequesterImpl<'_, LOCALIZER> {
    fn default() -> Self {
        LanguageRequesterImpl::<LOCALIZER>::new()
    }
}

impl<LOCALIZER> std::fmt::Debug for LanguageRequesterImpl<'_, LOCALIZER> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let listeners_debug: String = self
            .arc_listeners
            .iter()
            .map(|l| match l.upgrade() {
                Some(l) => format!("{l:p}"),
                None => "None".to_string(),
            })
            .collect::<Vec<String>>()
            .join(", ");
        write!(
            f,
            "LanguageRequesterImpl(listeners: {}, language_override: {:?})",
            listeners_debug, self.language_override,
        )
    }
}

/// A [LanguageRequester](LanguageRequester) for the desktop platform,
/// supporting windows, linux and mac. It uses
/// [locale_config](locale_config) to select the language based on the
/// system selected language.
///
/// ⚠️ *This API requires the following crate features to be activated: `desktop-requester`.*
#[cfg(feature = "desktop-requester")]
#[derive(Debug)]
pub struct DesktopLanguageRequester<'a, LOCALIZER> {
    implementation: LanguageRequesterImpl<'a, LOCALIZER>,
}

#[cfg(feature = "desktop-requester")]
impl<'a, LOCALIZER> LanguageRequester<'a> for DesktopLanguageRequester<'a, LOCALIZER>
where
    LOCALIZER: Localizer,
{
    type Localizer = LOCALIZER;
    fn requested_languages(&self) -> Vec<unic_langid::LanguageIdentifier> {
        DesktopLanguageRequester::<'a, LOCALIZER>::requested_languages()
    }

    fn add_listener(&mut self, listener: Weak<Self::Localizer>) {
        self.implementation.add_listener(listener)
    }

    fn add_listener_ref(&mut self, listener: &'a Self::Localizer) {
        self.implementation.add_listener_ref(listener)
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

    fn current_languages(&self) -> HashMap<String, unic_langid::LanguageIdentifier> {
        self.implementation.current_languages()
    }
}

#[cfg(feature = "desktop-requester")]
impl<LOCALIZER: Localizer> Default for DesktopLanguageRequester<'_, LOCALIZER> {
    fn default() -> Self {
        DesktopLanguageRequester::new()
    }
}

#[cfg(feature = "desktop-requester")]
impl<LOCALIZER: Localizer> DesktopLanguageRequester<'_, LOCALIZER> {
    /// Create a new `DesktopLanguageRequester`.
    pub fn new() -> Self {
        DesktopLanguageRequester {
            implementation: LanguageRequesterImpl::new(),
        }
    }

    /// The languages being requested by the operating
    /// system/environment according to the [locale_config] crate's
    /// implementation.
    pub fn requested_languages() -> Vec<unic_langid::LanguageIdentifier> {
        use locale_config::{LanguageRange, Locale};

        let current_locale = Locale::current();

        let ids: Vec<unic_langid::LanguageIdentifier> = current_locale
            .tags_for("messages")
            .filter_map(|tag: LanguageRange<'_>| match tag.to_string().parse() {
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
#[derive(Debug)]
pub struct WebLanguageRequester<'a, LOCALIZER> {
    implementation: LanguageRequesterImpl<'a, LOCALIZER>,
}

#[cfg(feature = "web-sys-requester")]
impl<LOCALIZER: Localizer> WebLanguageRequester<'_, LOCALIZER> {
    /// Create a new `WebLanguageRequester`.
    pub fn new() -> Self {
        WebLanguageRequester {
            implementation: LanguageRequesterImpl::new(),
        }
    }

    /// The languages currently being requested by the browser context.
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
impl<LOCALIZER: Localizer> Default for WebLanguageRequester<'_, LOCALIZER> {
    fn default() -> Self {
        WebLanguageRequester::new()
    }
}

#[cfg(feature = "web-sys-requester")]
impl<'a, LOCALIZER> LanguageRequester<'a> for WebLanguageRequester<'a, LOCALIZER>
where
    LOCALIZER: Localizer,
{
    type Localizer = LOCALIZER;
    fn requested_languages(&self) -> Vec<unic_langid::LanguageIdentifier> {
        Self::requested_languages()
    }

    fn add_listener(&mut self, listener: Weak<Self::Localizer>) {
        self.implementation.add_listener(listener)
    }

    fn add_listener_ref(&mut self, listener: &'a Self::Localizer) {
        self.implementation.add_listener_ref(listener)
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

    fn current_languages(&self) -> HashMap<String, unic_langid::LanguageIdentifier> {
        self.implementation.current_languages()
    }
}
