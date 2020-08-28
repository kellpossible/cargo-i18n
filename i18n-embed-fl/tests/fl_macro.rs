use i18n_embed::{
    fluent::{fluent_language_loader, FluentLanguageLoader},
    LanguageLoader,
};
use i18n_embed_fl::fl;
use rust_embed::RustEmbed;
use std::collections::HashMap;

#[derive(RustEmbed)]
#[folder = "i18n/"]
struct Localizations;

#[test]
fn with_args_hashmap() {
    let loader: FluentLanguageLoader = fluent_language_loader!();
    loader
        .load_languages(&Localizations, &[loader.fallback_language()])
        .unwrap();

    let mut args: HashMap<&str, &str> = HashMap::new();
    args.insert("name", "Bob");

    pretty_assertions::assert_eq!("Hello \u{2068}Bob\u{2069}!", fl!(loader, "hello-arg", args));
}

#[test]
fn with_loader_expr() {
    let loader = || {
        let loader: FluentLanguageLoader = fluent_language_loader!();
        loader
            .load_languages(&Localizations, &[loader.fallback_language()])
            .unwrap();
        loader
    };

    pretty_assertions::assert_eq!("Hello World!", fl!(loader(), "hello-world"));
}

#[test]
fn with_one_arg_lit() {
    let loader: FluentLanguageLoader = fluent_language_loader!();
    loader
        .load_languages(&Localizations, &[loader.fallback_language()])
        .unwrap();

    pretty_assertions::assert_eq!(
        "Hello \u{2068}Bob\u{2069}!",
        fl!(loader, "hello-arg", name = "Bob")
    );
}
