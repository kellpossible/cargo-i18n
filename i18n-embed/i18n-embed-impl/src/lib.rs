extern crate proc_macro;

use crate::proc_macro::TokenStream;

use quote::quote;

/// A procedural macro to implement the `I18nEmbed` trait on a struct.
///
/// ## Example
///
/// ```ignore
/// use rust_embed::RustEmbed;
/// use i18n_embed::I18nEmbed;
///
/// #[derive(RustEmbed, I18nEmbed)]
/// #[folder = "i18n"]
/// struct Localizations;
/// ```
#[proc_macro_derive(I18nEmbed)]
pub fn i18n_embed_derive(input: TokenStream) -> TokenStream {
    let ast: syn::DeriveInput = syn::parse(input).unwrap();

    let struct_name = &ast.ident;

    let gen = quote! {
        impl I18nEmbed for #struct_name {
        }
    };

    gen.into()
}

/// A procedural macro to create a new `GettextLanguageLoader` using
/// the current crate's `i18n.toml` configuration, and domain.
///
/// ⚠️ *This API requires the following crate features to be
/// activated: `gettext-system`.*
///
/// ## Example
///
/// ```ignore
/// use i18n_embed::gettext::{gettext_language_loader, GettextLanguageLoader};
/// let my_language_loader: GettextLanguageLoader = gettext_language_loader!();
/// ```
#[proc_macro]
#[cfg(feature = "gettext-system")]
pub fn gettext_language_loader(_: TokenStream) -> TokenStream {
    let config_file_path = std::path::PathBuf::from("i18n.toml");

    if !config_file_path.exists() {
        panic!(format!(
            "The i18n configuration file '{}' does not exist in the current working directory '{}'",
            config_file_path.to_string_lossy(),
            std::env::current_dir().unwrap().to_str().unwrap()
        ));
    }

    let config = i18n_config::I18nConfig::from_file(&config_file_path).unwrap_or_else(|err| {
        panic!(
            "gettext_language_loader!() had a problem reading config file '{0}': {1}",
            config_file_path.to_string_lossy(),
            err
        )
    });
    let fallback_language = &config.fallback_language;

    let gen = quote! {
        i18n_embed::gettext::GettextLanguageLoader::new(
            module_path!(),
            #fallback_language.parse().unwrap(),
        )
    };

    gen.into()
}

/// A procedural macro to create a new `FluentLanguageLoader` using
/// the current crate's `i18n.toml` configuration, and domain.
///
/// ⚠️ *This API requires the following crate features to be
/// activated: `fluent-system`.*
///
/// ## Example
///
/// ```ignore
/// use i18n_embed::fluent::{fluent_language_loader, FluentLanguageLoader};
/// let my_language_loader: FluentLanguageLoader = fluent_language_loader!();
/// ```
#[proc_macro]
#[cfg(feature = "fluent-system")]
pub fn fluent_language_loader(_: TokenStream) -> TokenStream {
    let config_file_path = std::path::PathBuf::from("i18n.toml");

    if !config_file_path.exists() {
        panic!(format!(
            "The i18n configuration file '{}' does not exist in the current working directory '{}'",
            config_file_path.to_string_lossy(),
            std::env::current_dir().unwrap().to_str().unwrap()
        ));
    }

    let config = i18n_config::I18nConfig::from_file(&config_file_path).unwrap_or_else(|err| {
        panic!(
            "fluent_language_loader!() had a problem reading config file '{0}': {1}",
            config_file_path.to_string_lossy(),
            err
        )
    });
    let fallback_language = &config.fallback_language;

    let gen = quote! {
        i18n_embed::fluent::FluentLanguageLoader::new(
            module_path!(),
            #fallback_language.parse().unwrap(),
        )
    };

    gen.into()
}
