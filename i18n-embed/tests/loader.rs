#[cfg(feature = "fluent-system")]
mod fluent {
    use i18n_embed::{fluent::FluentLanguageLoader, I18nEmbed, LanguageLoader};
    use rust_embed::RustEmbed;
    use unic_langid::LanguageIdentifier;

    #[derive(RustEmbed, I18nEmbed)]
    #[folder = "i18n/ftl"]
    struct Localizations;

    fn setup() {
        let _ = env_logger::try_init();
    }

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
        pretty_assertions::assert_eq!("Привет \u{2068}Tanya\u{2069}!", loader.get_args("only-ru-args", args));
    }
}

#[cfg(feature = "gettext-system")]
mod gettext {}
