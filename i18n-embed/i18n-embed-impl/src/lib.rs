extern crate proc_macro;

use crate::proc_macro::TokenStream;

use std::path::PathBuf;

use i18n_config::I18nConfig;

use quote::quote;
use syn;

/// A procedural macro to implement the `I18nEmbed` trait on a struct.
#[proc_macro_derive(I18nEmbed, attributes(config_file))]
pub fn i18n_embed_derive(input: TokenStream) -> TokenStream {
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
        impl I18nEmbed for #struct_name {
            fn src_locale() -> i18n_embed::LanguageIdentifier {
                #src_locale.parse().unwrap()
            }
        }
    };

    gen.into()
}
