use i18n_embed::{
    fluent::{fluent_language_loader, FluentLanguageLoader}, select, LanguageLoader
};
use rust_embed::RustEmbed;
use unic_langid::LanguageIdentifier;

#[derive(RustEmbed)]
#[folder = "./i18n/"]
struct Localizations;

lazy_static::lazy_static! {
    pub static ref LANGUAGE_LOADER: FluentLanguageLoader = {
        let loader: FluentLanguageLoader = fluent_language_loader!();

        loader
            .load_fallback_language(&Localizations)
            .expect("Error while loading fallback language");

        loader
    };
}

#[macro_export]
macro_rules! fl {
    ($message_id:literal) => {{
        i18n_embed_fl::fl!($crate::localize::LANGUAGE_LOADER, $message_id)
    }};

    ($message_id:literal, $($args:expr),*) => {{
        i18n_embed_fl::fl!($crate::localize::LANGUAGE_LOADER, $message_id, $($args), *)
    }};
}


pub fn available_languages() -> Vec<LanguageIdentifier> {
    (*LANGUAGE_LOADER)
        .available_languages(&Localizations)
        .expect("error while calculating available languages")

}


pub fn requested_languages() -> Vec<LanguageIdentifier> {
    i18n_embed::DesktopLanguageRequester::requested_languages()
}



pub fn select_language(requested_languages: &[LanguageIdentifier]) -> Vec<LanguageIdentifier> {
    select(&*LANGUAGE_LOADER, &Localizations, &requested_languages).unwrap()
}