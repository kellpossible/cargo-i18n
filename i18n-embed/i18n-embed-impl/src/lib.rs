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
///
/// The following struct level attributes available:
///
/// + (Optional) `#[dynamic(DynamicStructName)]` - also create and
///   derive a `I18nEmbedDyn` implementation with the specified
///   struct name.
#[proc_macro_derive(I18nEmbed, attributes(dynamic))]
pub fn i18n_embed_derive(input: TokenStream) -> TokenStream {
    let ast: syn::DeriveInput = syn::parse(input).unwrap();

    let dynamic_attribute: Option<&syn::Attribute> = ast.attrs.iter().find(|item: &&syn::Attribute| {
        match item.path.segments.first() {
            Some(segment) => {
                segment.ident.to_string() == "dynamic"
            },
            None => false
        }
    });

    let struct_name = &ast.ident;

    let mut gen1 = quote! {
        impl I18nEmbed for #struct_name {
        }
    };

    let gen_final = match dynamic_attribute {
        Some(attribute) => {
            let tokens: proc_macro::TokenStream = attribute.tokens.clone().into();

            let ast = syn::parse_macro_input!(tokens as syn::TypeParen);

            let dynamic_struct_name = match &*ast.elem {
                syn::Type::Path(path_type) => {
                    path_type.path.segments.first().expect("expected there to be at least one segment in dynamic(DynamicStructName) attribute").ident.clone()
                },
                _ => panic!("incorrectly formated dynamic(DynamicStructName) attribute")
            };

            let gen2 = quote! {
                #[derive(i18n_embed::I18nEmbedDyn)]
                struct #dynamic_struct_name;
            };

            gen1.extend(gen2);

            gen1
        },
        None => gen1
    };

    gen_final.into()
}

/// A procedural macro to implement the `I18nEmbedDyn` trait on a struct.
#[proc_macro_derive(I18nEmbedDyn, attributes(i18n_embed))]
pub fn dynamic_i18n_embed_derive(input: TokenStream) -> TokenStream {
    let ast: syn::DeriveInput = syn::parse(input).unwrap();

    let struct_name = &ast.ident;

    let gen = quote! {
        impl i18n_embed::I18nEmbedDyn for #struct_name {
            fn available_languages_dyn<'a>(&self, language_loader: &'a dyn i18n_embed::LanguageLoader) -> Result<Vec<i18n_embed::unic_langid::LanguageIdentifier>, i18n_embed::I18nEmbedError> {
                Translations::available_languages(language_loader)
            }
            fn load_language_file_dyn<'a>(&self, language_id: &i18n_embed::unic_langid::LanguageIdentifier, language_loader: &'a dyn i18n_embed::LanguageLoader) -> Result<(), i18n_embed::I18nEmbedError> {
                Translations::load_language_file(language_id, language_loader)
            }
        }
    };

    gen.into()
}

#[derive(Debug)]
struct ConstLocalizerInput {
    pub localizer_struct_name: syn::Ident,
    pub localizer_var_name: syn::Ident,
    pub embed_struct_name: syn::Ident,
    pub embed_var_name: syn::Ident,
    pub loader_struct_name: syn::Ident,
    pub loader_var_name: syn::Ident,
}

fn ident_from_nested_meta<'a>(nested_meta: &'a syn::NestedMeta) -> &'a syn::Ident {
    match nested_meta {
        syn::NestedMeta::Meta(meta) => {
            match meta {
                syn::Meta::Path(path) => {
                    &path.segments.first().unwrap().ident
                },
                _ => panic!()
            }
        },
        _ => panic!()
    }
}

fn get_ident_pair_with_path_ident<'a>(parsed: &'a Punctuated<syn::Meta, Comma>, ident: &str) -> (&'a syn::Ident, &'a syn::Ident) {
    let meta: &syn::Meta = parsed.iter().find(|item: &&syn::Meta| match item {
        syn::Meta::List(list) => {
            let path_segment: &syn::PathSegment = list.path.segments.first().unwrap();
            path_segment.ident.to_string() == ident
        },
        _ => panic!("unexpected item {0} in arguments for const_localizer!() macro")
    }).expect(&format!("expected to find meta {0}(StructName, VARIABLE_NAME) in const_localizer!() macro", ident));

    match meta {
        syn::Meta::List(list) => {
            let mut list_iter = list.nested.iter();
            let struct_name_meta: &syn::NestedMeta = list_iter.next().expect(&format!("expected there would be a StructName in the list {0}(StructName, VARIABLE_NAME) in const_localizer!() macro", ident));
            let struct_name_ident = ident_from_nested_meta(struct_name_meta);
            
            let variable_name_meta: &syn::NestedMeta  = list_iter.next().expect(&format!("expected there would be a VARIABLE_NAME in the list {0}(StructName, VARIABLE_NAME) in const_localizer!() macro", ident));
            let variable_name_ident = ident_from_nested_meta(variable_name_meta);

            if !list_iter.next().is_none() {
                panic!(format!("expected that there would only be two items in {0}(StructName, VARIABLE_NAME)", ident))
            }

            (struct_name_ident, variable_name_ident)
        },
        _ => panic!("unexpected item {0} in arguments for const_localizer!() macro")
    }
}

impl ParseMacroInput for ConstLocalizerInput {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> { 
        let parsed: Punctuated<syn::Meta, Comma> = Punctuated::parse_terminated(input)?;
        
        let (localizer_struct_name, localizer_var_name) = get_ident_pair_with_path_ident(&parsed, "localizer");
        let (embed_struct_name, embed_var_name) = get_ident_pair_with_path_ident(&parsed, "embed");
        let (loader_struct_name, loader_var_name) = get_ident_pair_with_path_ident(&parsed, "loader");
        
        Ok(ConstLocalizerInput {
            localizer_struct_name: localizer_struct_name.clone(),
            localizer_var_name: localizer_var_name.clone(),
            embed_struct_name: embed_struct_name.clone(),
            embed_var_name: embed_var_name.clone(),
            loader_struct_name: loader_struct_name.clone(),
            loader_var_name: loader_var_name.clone(),
        })
        
    }
    
}

/// A procedural macro to implement the `Localizer` trait on a struct.
#[proc_macro]
pub fn const_localizer(input: TokenStream) -> TokenStream {
    let args = syn::parse_macro_input!(input as ConstLocalizerInput);
    dbg!(&args);

    let localizer_struct_name = &args.localizer_struct_name;
    let localizer_var_name = &args.localizer_var_name;
    let embed_struct_name = &args.embed_struct_name;
    let embed_var_name = &args.embed_var_name;
    let loader_struct_name = &args.loader_struct_name;
    let loader_var_name = &args.loader_var_name;

    let gen = quote!{
        #[derive(i18n_embed::LanguageLoader)]
        #[cfg(feature = "localize")]
        struct #loader_struct_name;

        const #loader_var_name: #loader_struct_name = #loader_struct_name {};
        const #embed_var_name: #embed_struct_name = #embed_struct_name {};

        struct #localizer_struct_name<'a> {
            language_loader: &'a dyn i18n_embed::LanguageLoader,
            i18n_embed: &'a dyn i18n_embed::I18nEmbedDyn,
        }

        impl <'a> i18n_embed::Localizer<'a> for #localizer_struct_name<'a> {
            fn language_loader(&self) -> &'a dyn i18n_embed::LanguageLoader { self.language_loader }
            fn i18n_embed(&self) -> &'a dyn i18n_embed::I18nEmbedDyn { self.i18n_embed }
        }

        const #localizer_var_name: #localizer_struct_name = #localizer_struct_name {
            language_loader: &#loader_var_name,
            i18n_embed: &#embed_var_name,
        };
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