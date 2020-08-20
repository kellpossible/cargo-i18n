extern crate proc_macro;

use crate::proc_macro::TokenStream;

use i18n_config::I18nConfig;
use quote::quote;
use std::path::PathBuf;

/// A procedural macro to implement the `I18nEmbed` trait on a struct.
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
/// use i18n_embed::gettext_language_loader;
///
/// let my_language_loader: GettextLanguageLoader = gettext_language_loader!();
/// ```
#[proc_macro]
#[cfg(feature = "gettext-system")]
pub fn gettext_language_loader(_: TokenStream) -> TokenStream {
    let config_file_path = PathBuf::from("i18n.toml");

    if !config_file_path.exists() {
        panic!(format!(
            "The i18n configuration file '{}' does not exist in the current working directory '{}'",
            config_file_path.to_string_lossy(),
            std::env::current_dir().unwrap().to_str().unwrap()
        ));
    }

    let config = I18nConfig::from_file(&config_file_path).unwrap_or_else(|_| {
        panic!(
            "gettext_language_loader!() had a problem reading config file '{0}'",
            config_file_path.to_string_lossy()
        )
    });
    let fallback_locale = &config.fallback_locale;

    let gen = quote! {
        i18n_embed::gettext::GettextLanguageLoader::new(
            module_path!(),
            #fallback_locale.parse().unwrap(),
        )
    };

    gen.into()
}
