#![feature(proc_macro_diagnostic)]

use i18n_embed::{fluent::FluentLanguageLoader, LanguageLoader};
use proc_macro::TokenStream;
use quote::quote;
use syn::{parse::Parse, parse_macro_input};

struct TrMacroInput {
    fluent_loader: syn::Ident,
    message_id: syn::Lit,
}

impl Parse for TrMacroInput {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        let fluent_loader = input.parse()?;
        input.parse::<syn::Token![,]>()?;
        let localizations = input.parse()?;
        input.parse::<syn::Token![,]>()?;
        let message_id = input.parse()?;
        Ok(Self {
            fluent_loader,
            localizations,
            message_id,
        })
    }
}

lazy_static::lazy_static! {
    static ref LANGUAGE_LOADERS: dashmap::DashMap<String, i18n_embed::fluent::FluentLanguageLoader> =
        dashmap::DashMap::new();
}

#[proc_macro]
pub fn fl(input: TokenStream) -> TokenStream {
    let input: TrMacroInput = parse_macro_input!(input as TrMacroInput);

    let fluent_loader = input.fluent_loader;
    let message_id = input.message_id;

    match &message_id {
        syn::Lit::Str(message_id_str) => {
            let message_id_str = message_id_str.value();

            let manifest = find_crate::Manifest::new().expect("Error reading Cargo.toml");
            let current_crate_package = manifest.crate_package().expect("Error reading Cargo.toml");
            let domain = current_crate_package.name.clone();

            let loader = if let Some(loader) = LANGUAGE_LOADERS.get(&domain) {
                loader
            } else {
                let config_file_path = std::path::PathBuf::from("i18n.toml");
                let config =
                    i18n_config::I18nConfig::from_file(&config_file_path).unwrap_or_else(|err| {
                        panic!(
                        "gettext_language_loader!() had a problem reading config file {0:?}: {1}",
                        config_file_path,
                        err
                    )
                    });

                let loader = FluentLanguageLoader::new(
                    &domain,
                    config
                        .fallback_language
                        .parse()
                        .expect("unable to parse config fallback language"),
                );
                loader.load_languages(&[&config.fallback_language]);

                LANGUAGE_LOADERS.insert_and_get(domain.clone(), loader)
            };

            if !loader.has(&message_id_str) {
                message_id
                    .span()
                    .unstable()
                    .error(&format!(
                        "`message_id` of \"{0}\" does not exist in language \"{1}\"",
                        message_id_str,
                        loader.current_language(),
                    ))
                    .emit();
            }
        }
        unexpected_lit => {
            unexpected_lit
                .span()
                .unstable()
                .error("`message_id` should be a &'static str")
                .emit();
        }
    }

    let gen = quote! {
        #fluent_loader.get(#message_id)
    };

    gen.into()
}
