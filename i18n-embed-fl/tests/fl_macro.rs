use i18n_embed::{
    fluent::FluentLanguageLoader, unic_langid::LanguageIdentifier, LanguageLoader,
};
use i18n_embed_fl::fl;
use rust_embed::RustEmbed;

fn setup() {
    let _ = env_logger::try_init();
}

#[derive(RustEmbed)]
#[folder = "i18n/"]
struct Localizations;

#[test]
fn fl_macro() {
    setup();
    let en_us: LanguageIdentifier = "en-US".parse().unwrap();
    let loader = FluentLanguageLoader::new("i18n_embed_fl", en_us.clone());
    loader.load_languages(&Localizations, &[&en_us]).unwrap();
    pretty_assertions::assert_eq!(
        "Hello World!",
        fl!(loader, "hello-world")
    );
}
