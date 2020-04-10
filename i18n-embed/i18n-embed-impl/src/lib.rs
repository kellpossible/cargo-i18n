extern crate proc_macro;

use crate::proc_macro::TokenStream;

use std::path::PathBuf;
use i18n_config::I18nConfig;
use quote::quote;
use syn;
use syn::punctuated::Punctuated;
use syn::token::Comma;

use syn::parse_macro_input::ParseMacroInput;

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

/// A procedural macro to implement the `LanguageLoader` trait on a struct.
/// 
/// The following struct level attributes available:
///
/// + (Optional) `#[config_file = "i18n.toml"]` - path to i18n config
///   file relative to the crate root.
#[proc_macro_derive(LanguageLoader, attributes(config_file))]
#[cfg(feature = "gettext-system")]
pub fn language_loader_derive(input: TokenStream) -> TokenStream {
    let ast: syn::DeriveInput = syn::parse(input).unwrap();

    let struct_name = &ast.ident;

    let config_file_path = ast
        .attrs
        .iter()
        .find(|value| value.path.is_ident("config_file"))
        .map(|attr| {
            let meta: syn::Meta = attr.parse_meta().unwrap();
            let literal_value = match meta {
                syn::Meta::NameValue(ref data) => &data.lit,
                _ => panic!("#[derive(I18nEmbed)]'s attribute \"config_file\" should be formatted like: #[config_file = \"i18n.toml\"]")
            };

            let config_file_path = match literal_value {
                syn::Lit::Str(ref val) => PathBuf::from(val.clone().value()),
                _ => {
                panic!("#[derive(I18nEmbed)]'s attribute \"config_file\" value must be a string literal");
                }
            };

            config_file_path
        }).unwrap_or(PathBuf::from("i18n.toml"));

    if !config_file_path.exists() {
        panic!(format!(
            "#[derive(RustEmbed)] folder '{}' does not exist. cwd: '{}'",
            config_file_path.to_string_lossy(),
            std::env::current_dir().unwrap().to_str().unwrap()
        ));
    }

    let config = I18nConfig::from_file(&config_file_path).expect(&format!(
        "#[derive(RustEmbed)] had a problem reading config file {0}",
        config_file_path.to_string_lossy()
    ));
    let src_locale = &config.src_locale;

    let gen = quote! {
        impl LanguageLoader for #struct_name {
            fn load_language_file(&self, file: std::borrow::Cow<[u8]>) {
                let catalog = i18n_embed::gettext::Catalog::parse(&*file).expect("could not parse the catalog");
                i18n_embed::tr::set_translator!(catalog);
            }
        
            fn domain(&self) -> &'static str {
                i18n_embed::domain_from_module(module_path!())
            }

            fn src_locale(&self) -> i18n_embed::unic_langid::LanguageIdentifier {
                #src_locale.parse().unwrap()
            }

            fn language_file_name(&self) -> String {
                format!("{}.{}", self.domain(), "mo")
            }
        }
    };

    gen.into()
}