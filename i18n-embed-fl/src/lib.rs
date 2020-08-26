#![feature(proc_macro_diagnostic)]

use i18n_embed::{fluent::FluentLanguageLoader, LanguageLoader, FileSystemAssets};
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
        let message_id = input.parse()?;
        Ok(Self {
            fluent_loader,
            message_id,
        })
    }
}

struct DomainSpecificData {
    loader: FluentLanguageLoader,
    assets: FileSystemAssets,
}

lazy_static::lazy_static! {
    static ref DOMAINS: dashmap::DashMap<String, DomainSpecificData> =
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

            let domain_data = if let Some(domain_data) = DOMAINS.get(&domain) {
                domain_data
            } else {
                let config_file_path = std::path::PathBuf::from("i18n.toml");
                let config =
                    i18n_config::I18nConfig::from_file(&config_file_path).unwrap_or_else(|err| {
                        panic!(
                            "fl!() had a problem reading config file {0:?}: {1}",
                            config_file_path,
                            err
                        )
                    });

                let fluent_config = config.fluent.unwrap_or_else(|| {
                    panic!(
                        "fl!() had a problem parsing config file {0:?}: there is no `[fluent]` subsection.",
                        config_file_path
                    )
                });

                let assets = FileSystemAssets::new(fluent_config.assets_dir);

                let fallback_language: LanguageIdentifier = config
                        .fallback_language
                        .parse()
                        .expect("fl!() had a problem parsing config: unable to parse `fallback_language`");
                
                        let loader = FluentLanguageLoader::new(
                    &domain,
                    fallback_language.clone(),
                );

        
                loader.load_languages(&assets, &[&fallback_language]);

                let data = DomainSpecificData {
                    loader,
                    assets,
                };

                DOMAINS.insert_and_get(domain.clone(), data)
            };

            if !domain_data.loader.has(&message_id_str) {
                message_id
                    .span()
                    .unstable()
                    .error(&format!(
                        "`message_id` of \"{0}\" does not exist in language \"{1}\"",
                        message_id_str,
                        domain_data.loader.current_language(),
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
