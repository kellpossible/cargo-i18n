fn setup() {
    let _ = env_logger::try_init();
}

#[cfg(feature = "fluent-system")]
mod fluent {
    use super::setup;
    use i18n_embed::{fluent::FluentLanguageLoader, LanguageLoader};
    use rust_embed::RustEmbed;
    use unic_langid::LanguageIdentifier;

    #[derive(RustEmbed)]
    #[folder = "i18n/ftl"]
    struct Localizations;

    #[test]
    fn hello_world_en_us() {
        setup();
        let en_us: LanguageIdentifier = "en-US".parse().unwrap();
        let loader = FluentLanguageLoader::new("test", en_us.clone());
        loader.load_languages(&[&en_us], &Localizations).unwrap();
        pretty_assertions::assert_eq!("Hello World Localization!", loader.get("hello-world"));
    }

    #[test]
    fn fallback_en_gb_to_en_us() {
        setup();
        let en_us: LanguageIdentifier = "en-US".parse().unwrap();
        let en_gb: LanguageIdentifier = "en-GB".parse().unwrap();

        let loader = FluentLanguageLoader::new("test", en_us.clone());
        loader.load_languages(&[&en_gb], &Localizations).unwrap();
        pretty_assertions::assert_eq!("Hello World Localisation!", loader.get("hello-world"));
        pretty_assertions::assert_eq!("only US", loader.get("only-us"));
    }

    #[test]
    fn fallbacks_ru_to_en_gb_to_en_us() {
        setup();
        let ru: LanguageIdentifier = "ru".parse().unwrap();
        let en_us: LanguageIdentifier = "en-US".parse().unwrap();
        let en_gb: LanguageIdentifier = "en-GB".parse().unwrap();

        let loader = FluentLanguageLoader::new("test", en_us.clone());
        loader
            .load_languages(&[&ru, &en_gb], &Localizations)
            .unwrap();
        pretty_assertions::assert_eq!("Привет Мир Локализация!", loader.get("hello-world"));
        pretty_assertions::assert_eq!("only GB", loader.get("only-gb"));
        pretty_assertions::assert_eq!("only US", loader.get("only-us"));
        pretty_assertions::assert_eq!("только русский", loader.get("only-ru"));
    }

    #[test]
    fn args_fallback_ru_to_en_us() {
        setup();
        let en_us: LanguageIdentifier = "en-US".parse().unwrap();
        let ru: LanguageIdentifier = "ru".parse().unwrap();

        let loader = FluentLanguageLoader::new("test", en_us.clone());
        loader.load_languages(&[&ru], &Localizations).unwrap();

        let args = maplit::hashmap! {
            "userName" => "Tanya"
        };
        pretty_assertions::assert_eq!(
            "Привет \u{2068}Tanya\u{2069}!",
            loader.get_args("only-ru-args", args)
        );
    }

    #[test]
    fn has() {
        setup();
        let en_us: LanguageIdentifier = "en-US".parse().unwrap();
        let ru: LanguageIdentifier = "ru".parse().unwrap();

        let loader = FluentLanguageLoader::new("test", en_us.clone());
        loader.load_languages(&[&ru], &Localizations).unwrap();

        assert!(loader.has("only-ru-args"));
        assert!(loader.has("only-us"));
        assert!(!loader.has("non-existent-message"))
    }
}

#[cfg(feature = "gettext-system")]
mod gettext {
    use super::setup;
    use i18n_embed::{gettext::GettextLanguageLoader, LanguageLoader};
    use rust_embed::RustEmbed;
    use tr::internal::with_translator;
    use unic_langid::LanguageIdentifier;

    /// Custom version of the tr! macro function, without the runtime
    /// formatting, with the module set to `i18n_embed` where the
    /// strings were originally extracted from.
    fn tr(msgid: &str) -> String {
        with_translator("i18n_embed", |t| t.translate(msgid, None).to_string())
    }

    #[derive(RustEmbed)]
    #[folder = "i18n/mo"]
    struct Localizations;

    lazy_static::lazy_static! {
        static ref LOADER: GettextLanguageLoader = GettextLanguageLoader::new("i18n_embed", "en".parse().unwrap());
    }

    #[test]
    fn only_en() {
        setup();

        let ru: LanguageIdentifier = "ru".parse().unwrap();
        let en: LanguageIdentifier = "en".parse().unwrap();

        LOADER.load_languages(&[&ru], &Localizations).unwrap();

        // It should replace the ru with en
        LOADER.load_languages(&[&en], &Localizations).unwrap();

        pretty_assertions::assert_eq!("only en", tr("only en"));
        pretty_assertions::assert_eq!("only ru", tr("only ru"));
    }

    #[test]
    fn fallback_ru_to_en() {
        setup();

        let ru: LanguageIdentifier = "ru".parse().unwrap();

        assert!(Localizations::get("ru/i18n_embed.mo").is_some());
        LOADER.load_languages(&[&ru], &Localizations).unwrap();

        pretty_assertions::assert_eq!("только ру", tr("only ru"));
        pretty_assertions::assert_eq!("only en", tr("only en"));
    }
}
