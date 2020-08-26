#![feature(proc_macro_diagnostic)]

use i18n_embed::{fluent::FluentLanguageLoader, LanguageLoader, FileSystemAssets};
use proc_macro::TokenStream;
use quote::quote;
use syn::{parse::Parse, parse_macro_input, spanned::Spanned};
use unic_langid::LanguageIdentifier;
use std::{collections::HashMap, path::Path};

#[derive(Debug)]
enum FlArgs {
    HashMap(syn::Ident),
    KeyValuePairs(HashMap<syn::LitStr, Box<syn::Expr>>),
    None,
}

impl Parse for FlArgs {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        if !input.is_empty() {
            input.parse::<syn::Token![,]>()?;

            let lookahead = input.fork();
            if let Ok(_) = lookahead.parse::<syn::Ident>() {
                if lookahead.is_empty() {
                    let hash_map = input.parse()?;
                    return Ok(FlArgs::HashMap(hash_map))
                }
            }

            let mut args_map: HashMap<syn::LitStr, Box<syn::Expr>> = HashMap::new();

            while let Ok(expr) = input.parse::<syn::ExprAssign>() {
                let argument_name_ident_opt = match &*expr.left {
                    syn::Expr::Path(path) => {
                        path.path.get_ident()
                    }
                    _ => {
                        None
                    }
                };

                let argument_name_ident = match argument_name_ident_opt {
                    Some(ident) => ident,
                    None => {
                        return Err(syn::Error::new(expr.left.span(), "unable to parse as an identifier"))
                    }
                }.clone();

                let argument_name_string = argument_name_ident.to_string();
                let argument_name_lit_str = syn::LitStr::new(&argument_name_string, argument_name_ident.span());

                let argument_value = expr.right;

                args_map.insert(argument_name_lit_str, argument_value);

                // parse the next comma if there is one
                let _result = input.parse::<syn::Token![,]>();
            }

            if args_map.is_empty() {
                let span = match input.fork().parse::<syn::Expr>() {
                    Ok(expr) => {
                        expr.span()
                    }
                    Err(_) => {
                        input.span()
                    }
                };
                Err(syn::Error::new(span, "unable to parse args input"))
            } else {
                Ok(FlArgs::KeyValuePairs(args_map))
            }
        } else {
            Ok(FlArgs::None)
        }
    }
}

struct FlMacroInput {
    fluent_loader: syn::Ident,
    message_id: syn::Lit,
    args: FlArgs, 
}

impl Parse for FlMacroInput {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        let fluent_loader = input.parse()?;
        input.parse::<syn::Token![,]>()?;
        let message_id = input.parse()?;
        
        let args = input.parse()?;
        
        Ok(Self {
            fluent_loader,
            message_id,
            args,
        })
    }
}

struct DomainSpecificData {
    loader: FluentLanguageLoader,
    _assets: FileSystemAssets,
}

lazy_static::lazy_static! {
    static ref DOMAINS: dashmap::DashMap<String, DomainSpecificData> =
        dashmap::DashMap::new();
}

/// A macro to obtain localized messages, and check the `message_id`
/// at compile time.
#[proc_macro]
pub fn fl(input: TokenStream) -> TokenStream {
    let input: FlMacroInput = parse_macro_input!(input as FlMacroInput);

    let fluent_loader = input.fluent_loader;
    let message_id = input.message_id;

    let manifest = find_crate::Manifest::new().expect("Error reading Cargo.toml");
            
    let current_crate_package = manifest.crate_package().expect("Error reading Cargo.toml");
    let domain = current_crate_package.name.clone();

    let domain_data = if let Some(domain_data) = DOMAINS.get(&domain) {
        domain_data
    } else {
        let crate_dir = std::env::var_os("CARGO_MANIFEST_DIR").unwrap_or_else(|| {
            panic!("fl!() had a problem reading `CARGO_MANIFEST_DIR` \
            environment variable")
        });
        let config_file_path = Path::new(&crate_dir).join("i18n.toml");

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
                "fl!() had a problem parsing config file {0:?}: \
                there is no `[fluent]` subsection.",
                config_file_path
            )
        });

        let assets_dir = Path::new(&crate_dir).join(fluent_config.assets_dir);
        let assets = FileSystemAssets::new(assets_dir);

        let fallback_language: LanguageIdentifier = config
                .fallback_language
                .parse()
                .expect("fl!() had a problem parsing config: unable to parse `fallback_language`");
        
                let loader = FluentLanguageLoader::new(
            &domain,
            fallback_language.clone(),
        );


        loader.load_languages(&assets, &[&fallback_language]).unwrap_or_else(|err| {
            match err {
                i18n_embed::I18nEmbedError::LanguageNotAvailable(file, language_id) => {
                    if fallback_language != language_id {
                        panic!(
                            "fl!() encountered an unexpected problem, \
                            the language being loaded (\"{0}\") is not the \
                            `fallback_language` (\"{1}\")",
                            language_id,
                            fallback_language
                        )
                    }
                    panic!(
                        "fl!() was unable to load the localization \
                        file for the `fallback_language` (\"{0}\"): {1}",
                        fallback_language,
                        file,
                    )
                }
                _ => {
                    panic!(
                        "fl!() had an unexpected problem while \
                        loading language \"{0}\": {1}",
                        fallback_language,
                        err
                    )   
                }
            }
        });

        let data = DomainSpecificData {
            loader,
            _assets: assets,
        };

        DOMAINS.insert_and_get(domain.clone(), data)
    };

    match &message_id {
        syn::Lit::Str(message_id_str) => {
            let message_id_str = message_id_str.value();
            
            if !domain_data.loader.has(&message_id_str) {
                message_id
                    .span()
                    .unstable()
                    .error(&format!(
                        "fl!() `message_id` validation failed. `message_id` \
                        of \"{0}\" does not exist in the `fallback_language` (\"{1}\")",
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

    let gen = match input.args {
        FlArgs::HashMap(args_hash_map) => {
            quote! {
                #fluent_loader.get_args(#message_id, #args_hash_map)
            }
        }
        FlArgs::None => {
            quote! {
                #fluent_loader.get(#message_id)
            }
        }
        FlArgs::KeyValuePairs(pairs) => {
            let mut arg_assignments = proc_macro2::TokenStream::default();

            for (key, value) in pairs {
                arg_assignments = quote! {
                    #arg_assignments
                    args.insert(#key, #value.into());
                }
            }

            let gen = quote!{ 
                #fluent_loader.get_args_concrete(
                    #message_id,
                    {
                        let mut args = std::collections::HashMap::new();
                        #arg_assignments
                        args
                    })
            };

            gen
        }
    };

    gen.into()
}
