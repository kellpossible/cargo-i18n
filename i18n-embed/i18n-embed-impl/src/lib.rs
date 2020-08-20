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

/// A procedural macro to create a struct and implement the `LanguageLoader` trait on it.
///
/// ## Example
///
/// ```ignore
/// use i18n_embed::language_loader;
///
/// language_loader!(MyLanguageLoader);
/// let my_language_loader = MyLanguageLoader::new();
/// ```
#[proc_macro]
#[cfg(feature = "gettext-system")]
pub fn language_loader(input: TokenStream) -> TokenStream {
    let struct_name = syn::parse_macro_input!(input as syn::Ident);

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
            "language_loader!() had a problem reading config file '{0}'",
            config_file_path.to_string_lossy()
        )
    });
    let fallback_locale = &config.fallback_locale;

    let gen = quote! {
        struct #struct_name {
            current_language: std::sync::RwLock<i18n_embed::unic_langid::LanguageIdentifier>,
        }

        impl #struct_name {
            pub fn new() -> #struct_name {
                #struct_name {
                    current_language: std::sync::RwLock::new(#fallback_locale.parse().unwrap()),
                }
            }
        }

        impl i18n_embed::LanguageLoader for #struct_name {
            /// Set the current language to the language corresponding with the specified `language_id`,
            /// using the resources packaged in the `i18n_embed`.
            fn load_language(
                &self,
                language_id: &i18n_embed::unic_langid::LanguageIdentifier,
                i18n_embed: &dyn i18n_embed::I18nEmbedDyn,
            ) -> Result<(), i18n_embed::I18nEmbedError> {
                if language_id == &self.fallback_locale() {
                    self.load_fallback_locale();
                    return Ok(());
                }

                let language_id_string = language_id.to_string();

                let file_path = format!("{}/{}", language_id_string, self.language_file_name());

                let f = i18n_embed
                    .get_dyn(file_path.as_ref())
                    .ok_or(i18n_embed::I18nEmbedError::LanguageNotAvailable(language_id_string))?;

                let catalog = i18n_embed::gettext::Catalog::parse(&*f).expect("could not parse the catalog");
                i18n_embed::tr::set_translator!(catalog);
                *(self.current_language.write().unwrap()) = language_id.clone();

                Ok(())
            }
            
            fn load_fallback_locale(&self) {
                let catalog = i18n_embed::gettext::Catalog::empty();
                i18n_embed::tr::set_translator!(catalog);
                *(self.current_language.write().unwrap()) = self.fallback_locale();
            }

            fn domain(&self) -> &'static str {
                i18n_embed::domain_from_module(module_path!())
            }

            fn fallback_locale(&self) -> i18n_embed::unic_langid::LanguageIdentifier {
                #fallback_locale.parse().unwrap()
            }

            fn language_file_name(&self) -> String {
                format!("{}.{}", self.domain(), "mo")
            }

            /// Get the language which is currently loaded for this loader.
            fn current_language(&self) -> i18n_embed::unic_langid::LanguageIdentifier {
                self.current_language.read().unwrap().clone()
            }
        }
    };

    gen.into()
}
