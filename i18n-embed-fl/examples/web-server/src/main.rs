use i18n_embed::{
    fluent::{fluent_language_loader, FluentLanguageLoader, NegotiationStrategy},
    LanguageLoader,
};
use i18n_embed_fl::fl;
use rust_embed::RustEmbed;

#[derive(RustEmbed)]
#[folder = "i18n/"]
struct Localizations;

fn main() {
    let loader: FluentLanguageLoader = fluent_language_loader!();
    loader.load_available_languages(&Localizations).unwrap();

    println!(
        "available languages: {:?}",
        loader.available_languages(&Localizations).unwrap()
    );

    println!(
        "requested [en-US], response: {}",
        hande_request(&loader, &[&"en-US".parse().unwrap()])
    );
    println!(
        "requested [ka-GE], response: {}",
        hande_request(&loader, &[&"ka-GE".parse().unwrap()])
    );
    println!(
        "requested [en-UK], response: {}",
        hande_request(&loader, &[&"en-UK".parse().unwrap()])
    );
    println!(
        "requested [de-AT], response: {}",
        hande_request(&loader, &[&"de-AT".parse().unwrap()])
    );
    println!(
        "requested [ru-RU], response: {}",
        hande_request(
            &loader,
            &[&"ru-RU".parse().unwrap(), &"de-DE".parse().unwrap()]
        )
    );
}

fn hande_request(
    loader: &FluentLanguageLoader,
    requested_languages: &[&unic_langid::LanguageIdentifier],
) -> String {
    let loader =
        loader.select_languages_negotiate(requested_languages, NegotiationStrategy::Filtering);
    let message = fl!(&loader, "hello-world");
    format!("<html><body><h1>{message}</h1></body></html>")
}
