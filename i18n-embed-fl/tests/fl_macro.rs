use i18n_embed::{I18nEmbed, fluent::FluentLanguageLoader, unic_langid::LanguageIdentifier, LanguageLoader};
use rust_embed::RustEmbed;
use i18n_embed_fl::fl;

fn setup() {
    let _ = env_logger::try_init();
}

#[derive(RustEmbed, I18nEmbed)]
#[folder = "../i18n-embed/i18n"]
struct Localizations;

#[test]
fn fl_macro() {
    setup();
    let en_us: LanguageIdentifier = "en-US".parse().unwrap();
    let loader = FluentLanguageLoader::new("test", en_us.clone());
    loader.load_languages(&[&en_us], &Localizations).unwrap();
    pretty_assertions::assert_eq!("Hello World Localization!", fl!(loader, Localizations, "only-us"));
}