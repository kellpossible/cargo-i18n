use i18n_embed::{
    fluent::{fluent_language_loader, FluentLanguageLoader},
    DefaultLocalizer, I18nAssets, LanguageLoader, Localizer, RustEmbedNotifyAssets,
};
use i18n_embed_fl::fl;
use once_cell::sync::Lazy;
use rust_embed::RustEmbed;

#[derive(RustEmbed)]
#[folder = "i18n/"]
pub struct LocalizationsEmbed;

pub static LOCALIZATIONS: Lazy<RustEmbedNotifyAssets<LocalizationsEmbed>> = Lazy::new(|| {
    RustEmbedNotifyAssets::new(std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("i18n/"))
});

static LANGUAGE_LOADER: Lazy<FluentLanguageLoader> = Lazy::new(|| {
    let loader: FluentLanguageLoader = fluent_language_loader!();

    // Load the fallback langauge by default so that users of the
    // library don't need to if they don't care about localization.
    loader
        .load_fallback_language(&*LOCALIZATIONS)
        .expect("Error while loading fallback language");

    loader
});

macro_rules! fl {
    ($message_id:literal) => {{
        i18n_embed_fl::fl!($crate::LANGUAGE_LOADER, $message_id)
    }};

    ($message_id:literal, $($args:expr),*) => {{
        i18n_embed_fl::fl!($crate::LANGUAGE_LOADER, $message_id, $($args), *)
    }};
}

/// Get the hello world statement in whatever the currently selected
/// localization is.
pub fn hello_world() -> String {
    fl!("hello-world")
}

// Get the `Localizer` to be used for localizing this library.
pub fn localizer(
) -> DefaultLocalizer<'static, RustEmbedNotifyAssets<LocalizationsEmbed>, FluentLanguageLoader> {
    DefaultLocalizer::new(&LOCALIZATIONS, &*LANGUAGE_LOADER)
}
