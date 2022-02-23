#[cfg(any(feature = "fluent-system", feature = "gettext-system"))]
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
        loader.load_languages(&Localizations, &[&en_us]).unwrap();
        pretty_assertions::assert_eq!("Hello World Localization!", loader.get("hello-world"));
    }

    #[test]
    fn fallback_en_gb_to_en_us() {
        setup();
        let en_us: LanguageIdentifier = "en-US".parse().unwrap();
        let en_gb: LanguageIdentifier = "en-GB".parse().unwrap();

        let loader = FluentLanguageLoader::new("test", en_us.clone());
        loader.load_languages(&Localizations, &[&en_gb]).unwrap();
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
            .load_languages(&Localizations, &[&ru, &en_gb])
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
        loader.load_languages(&Localizations, &[&ru]).unwrap();

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
        loader.load_languages(&Localizations, &[&ru]).unwrap();

        assert!(loader.has("only-ru-args"));
        assert!(loader.has("only-us"));
        assert!(!loader.has("non-existent-message"))
    }

    #[test]
    fn bidirectional_isolation_off() {
        setup();
        let en_us: LanguageIdentifier = "en-US".parse().unwrap();
        let loader = FluentLanguageLoader::new("test", en_us.clone());
        loader.load_languages(&Localizations, &[&en_us]).unwrap();
        loader.set_use_isolating(false);
        let args = maplit::hashmap! {
            "thing" => "thing"
        };
        let msg = loader.get_args("isolation-chars", args);
        assert_eq!("inject a thing here", msg);
    }

    #[test]
    fn bidirectional_isolation_on() {
        setup();
        let en_us: LanguageIdentifier = "en-US".parse().unwrap();
        let loader = FluentLanguageLoader::new("test", en_us.clone());
        loader.load_languages(&Localizations, &[&en_us]).unwrap();
        let args = maplit::hashmap! {
            "thing" => "thing"
        };
        let msg = loader.get_args("isolation-chars", args);
        assert_eq!("inject a \u{2068}thing\u{2069} here", msg);
    }

    #[test]
    fn multiline_lf() {
        setup();
        let en_us: LanguageIdentifier = "en-US".parse().unwrap();
        let loader = FluentLanguageLoader::new("test", en_us.clone());
        loader.load_languages(&Localizations, &[&en_us]).unwrap();

        let msg = loader.get("multi-line");
        assert_eq!(
            "This is a multi-line message.\n\n\
            This is a multi-line message.\n\n\
            Finished!",
            msg
        );
    }

    #[test]
    fn multiline_crlf() {
        setup();
        let ru: LanguageIdentifier = "ru".parse().unwrap();
        let loader = FluentLanguageLoader::new("test", ru.clone());
        loader.load_languages(&Localizations, &[&ru]).unwrap();

        let msg = loader.get("multi-line");
        assert_eq!(
            "Это многострочное сообщение.\n\n\
            Это многострочное сообщение.\n\n\
            Законченный!",
            msg
        );
    }

    #[test]
    fn multiline_arguments_lf() {
        setup();
        let en_us: LanguageIdentifier = "en-US".parse().unwrap();
        let loader = FluentLanguageLoader::new("test", en_us.clone());
        loader.load_languages(&Localizations, &[&en_us]).unwrap();

        let args = maplit::hashmap! {
            "argOne" => "1",
            "argTwo" => "2",
        };

        let msg = loader.get_args("multi-line-args", args);
        assert_eq!(
            "This is a multiline message with arguments.\n\n\
            \u{2068}1\u{2069}\n\n\
            This is a multiline message with arguments.\n\n\
            \u{2068}2\u{2069}\n\n\
            Finished!",
            msg
        );
    }

    #[test]
    fn multiline_arguments_crlf() {
        setup();
        let ru: LanguageIdentifier = "ru".parse().unwrap();
        let loader = FluentLanguageLoader::new("test", ru.clone());
        loader.load_languages(&Localizations, &[&ru]).unwrap();

        let args = maplit::hashmap! {
            "argOne" => "1",
            "argTwo" => "2",
        };

        let msg = loader.get_args("multi-line-args", args);
        assert_eq!(
            "Это многострочное сообщение с параметрами.\n\n\
            \u{2068}1\u{2069}\n\n\
            Это многострочное сообщение с параметрами.\n\n\
            \u{2068}2\u{2069}\n\n\
            Законченный!",
            msg
        );
    }

    #[test]
    fn get_lang_default_fallback() {
        setup();
        let ru: LanguageIdentifier = "ru".parse().unwrap();
        let en_gb: LanguageIdentifier = "en-GB".parse().unwrap();
        let en_us: LanguageIdentifier = "en-US".parse().unwrap();
        let loader = FluentLanguageLoader::new("test", en_us);

        loader
            .load_languages(&Localizations, &[&ru, &en_gb])
            .unwrap();

        let msg = loader.lang(&[&ru]).get("only-ru");
        assert_eq!("только русский", msg);

        let msg = loader.lang(&[&ru]).get("only-gb");
        assert_eq!("only GB (US Version)", msg);
    }

    #[test]
    fn get_lang_args_default_fallback() {
        setup();
        let ru: LanguageIdentifier = "ru".parse().unwrap();
        let en_gb: LanguageIdentifier = "en-GB".parse().unwrap();
        let en_us: LanguageIdentifier = "en-US".parse().unwrap();
        let loader = FluentLanguageLoader::new("test", en_us);

        loader
            .load_languages(&Localizations, &[&ru, &en_gb])
            .unwrap();

        let args = maplit::hashmap! {
            "argOne" => "1",
            "argTwo" => "2",
        };

        let msg = loader.lang(&[&ru]).get_args("multi-line-args", args);
        assert_eq!(
            "Это многострочное сообщение с параметрами.\n\n\
            \u{2068}1\u{2069}\n\n\
            Это многострочное сообщение с параметрами.\n\n\
            \u{2068}2\u{2069}\n\n\
            Законченный!",
            msg
        );
    }

    #[test]
    fn get_lang_custom_fallback() {
        setup();
        let ru: LanguageIdentifier = "ru".parse().unwrap();
        let en_gb: LanguageIdentifier = "en-GB".parse().unwrap();
        let en_us: LanguageIdentifier = "en-US".parse().unwrap();
        let loader = FluentLanguageLoader::new("test", en_us);

        loader
            .load_languages(&Localizations, &[&ru, &en_gb])
            .unwrap();

        let msg = loader.lang(&[&ru, &en_gb]).get("only-gb");
        assert_eq!("only GB", msg);

        let msg = loader.lang(&[&ru, &en_gb]).get("only-us");
        assert_eq!("only US", msg);
    }

    #[test]
    fn get_lang_args_custom_fallback() {
        setup();
        let ru: LanguageIdentifier = "ru".parse().unwrap();
        let en_gb: LanguageIdentifier = "en-GB".parse().unwrap();
        let en_us: LanguageIdentifier = "en-US".parse().unwrap();
        let loader = FluentLanguageLoader::new("test", en_us);

        loader
            .load_languages(&Localizations, &[&ru, &en_gb])
            .unwrap();

        let args = maplit::hashmap! {
            "userName" => "username",
        };

        let msg = loader.lang(&[&ru]).get_args("only-gb-args", args.clone());
        assert_eq!("Hello \u{2068}username\u{2069}! (US Version)", msg);

        let msg = loader
            .lang(&[&ru, &en_gb])
            .get_args("only-gb-args", args.clone());
        assert_eq!("Hello \u{2068}username\u{2069}!", msg);
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

    #[test]
    fn only_en() {
        setup();

        let loader = GettextLanguageLoader::new("i18n_embed", "en".parse().unwrap());

        let ru: LanguageIdentifier = "ru".parse().unwrap();
        let en: LanguageIdentifier = "en".parse().unwrap();

        loader.load_languages(&Localizations, &[&ru]).unwrap();

        // It should replace the ru with en
        loader.load_languages(&Localizations, &[&en]).unwrap();

        pretty_assertions::assert_eq!("only en", tr("only en"));
        pretty_assertions::assert_eq!("only ru", tr("only ru"));
    }

    #[test]
    fn fallback_ru_to_en() {
        setup();

        let loader = GettextLanguageLoader::new("i18n_embed", "en".parse().unwrap());

        let ru: LanguageIdentifier = "ru".parse().unwrap();

        assert!(Localizations::get("ru/i18n_embed.mo").is_some());
        loader.load_languages(&Localizations, &[&ru]).unwrap();

        pretty_assertions::assert_eq!("только ру", tr("only ru"));
        pretty_assertions::assert_eq!("only en", tr("only en"));
    }
}
