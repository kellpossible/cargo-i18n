#![feature(proc_macro_diagnostic)]
use proc_macro::TokenStream;

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
    let manifest = find_crate::Manifest::new().expect("Error reading Cargo.toml");
    let current_crate_package = manifest.crate_package().expect("Error reading Cargo.toml");

    // Special case for when this macro is invoked in i18n-embed tests/docs
    let i18n_embed_crate_name = if current_crate_package.name == "i18n_embed" {
        "i18n_embed".to_string()
    } else {
        manifest
            .find(|s| s == "i18n-embed")
            .expect("i18n-embed should be an active dependency in your Cargo.toml")
            .name
    };

    let i18n_embed_crate_ident =
        syn::Ident::new(&i18n_embed_crate_name, proc_macro2::Span::call_site());

    let config_file_path = std::path::PathBuf::from("i18n.toml");

    let config = i18n_config::I18nConfig::from_file(&config_file_path).unwrap_or_else(|err| {
        panic!(
            "gettext_language_loader!() had a problem reading config file {0:?}: {1}",
            config_file_path, err
        )
    });

    let fallback_language = &config.fallback_language;

    let gen = quote::quote! {
        #i18n_embed_crate_ident::gettext::GettextLanguageLoader::new(
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
    let manifest = find_crate::Manifest::new().expect("Error reading Cargo.toml");
    let current_crate_package = manifest.crate_package().expect("Error reading Cargo.toml");
    let domain = syn::Ident::new(&current_crate_package.name, proc_macro2::Span::call_site());

    // Special case for when this macro is invoked in i18n-embed tests/docs
    let i18n_embed_crate_name = if current_crate_package.name == "i18n_embed" {
        "i18n_embed".to_string()
    } else {
        manifest
            .find(|s| s == "i18n-embed")
            .expect("i18n-embed should be an active dependency in your Cargo.toml")
            .name
    };

    let i18n_embed_crate_ident =
        syn::Ident::new(&i18n_embed_crate_name, proc_macro2::Span::call_site());

    let config_file_path = std::path::PathBuf::from("i18n.toml");

    let config = i18n_config::I18nConfig::from_file(&config_file_path).unwrap_or_else(|err| {
        panic!(
            "fluent_language_loader!() had a problem reading config file {0:?}: {1}",
            config_file_path, err
        )
    });
    let fallback_language = &config.fallback_language;

    let gen = quote::quote! {
        #i18n_embed_crate_ident::fluent::FluentLanguageLoader::new(
            #domain,
            #fallback_language.parse().unwrap(),
        )
    };

    gen.into()
}
