use i18n_embed::{fluent::FluentLanguageLoader, unic_langid::LanguageIdentifier, LanguageLoader};
use i18n_embed_fl::fl;
use rust_embed::RustEmbed;

#[derive(RustEmbed)]
#[folder = "i18n/"]
struct Localizations;

#[test]
fn no_args() {
    let en_us: LanguageIdentifier = "en-US".parse().unwrap();
    let loader = FluentLanguageLoader::new("i18n_embed_fl", en_us.clone());
    loader.load_languages(&Localizations, &[&en_us]).unwrap();
    pretty_assertions::assert_eq!("Hello World!", fl!(loader, "hello-world"));
}

#[test]
fn with_args_hashmap() {
    let en_us: LanguageIdentifier = "en-US".parse().unwrap();
    let loader = FluentLanguageLoader::new("i18n_embed_fl", en_us.clone());
    loader.load_languages(&Localizations, &[&en_us]).unwrap();

    let args = maplit::hashmap! {
        "name" => "Bob"
    };

    pretty_assertions::assert_eq!("Hello \u{2068}Bob\u{2069}!", fl!(loader, "hello-arg", args));
}

#[test]
fn with_one_arg_lit() {
    let en_us: LanguageIdentifier = "en-US".parse().unwrap();
    let loader = FluentLanguageLoader::new("i18n_embed_fl", en_us.clone());
    loader.load_languages(&Localizations, &[&en_us]).unwrap();

    pretty_assertions::assert_eq!(
        "Hello \u{2068}Bob\u{2069}!",
        fl!(loader, "hello-arg", name = "Bob")
    );
}

#[test]
fn with_one_arg_expr() {
    let en_us: LanguageIdentifier = "en-US".parse().unwrap();
    let loader = FluentLanguageLoader::new("i18n_embed_fl", en_us.clone());
    loader.load_languages(&Localizations, &[&en_us]).unwrap();

    pretty_assertions::assert_eq!(
        "Hello \u{2068}Bob 23\u{2069}!",
        fl!(loader, "hello-arg", name = format!("Bob {}", 23))
    );
}

// #[test]
// fn with_two_args_fail() {
//     let en_us: LanguageIdentifier = "en-US".parse().unwrap();
//     let loader = FluentLanguageLoader::new("i18n_embed_fl", en_us.clone());
//     loader.load_languages(&Localizations, &[&en_us]).unwrap();

//     pretty_assertions::assert_eq!(
//         "Hello \u{2068}{$name1}\u{2069} and \u{2068}{$name2}\u{2069}!",
//         fl!(loader, "hello-arg-2", name1 = "Bob", name2 = "James", name = "Bob")
//     );
// }

#[test]
fn with_two_args() {
    let en_us: LanguageIdentifier = "en-US".parse().unwrap();
    let loader = FluentLanguageLoader::new("i18n_embed_fl", en_us.clone());
    loader.load_languages(&Localizations, &[&en_us]).unwrap();

    pretty_assertions::assert_eq!(
        "Hello \u{2068}Bob\u{2069} and \u{2068}James\u{2069}!",
        fl!(loader, "hello-arg-2", name1 = "Bob", name2 = "James")
    );
}
