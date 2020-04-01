extern crate proc_macro;
extern crate proc_macro2;

use crate::proc_macro::TokenStream;

use proc_macro2::{Ident, Span};
use quote::quote;
use syn;


#[proc_macro]
pub fn i18n_embed(input: TokenStream) -> TokenStream {
    // let input_parsed = syn::parse_macro_input!(input as syn::MetaList);
    let gen = quote!{
        use rust_embed;
        #[derive(rust_embed::RustEmbed)]
        #[folder = "i18n/mo"]
        struct Translations;

        impl Translations {
            pub fn available_languages() -> Vec<String> {
                use std::path::{Path, Component};
                use std::collections::HashSet;

                let mut languages: Vec<String> = Self::iter()
                    .map(|filename_cow| filename_cow.to_string())
                    .filter_map(|filename| {
                        let path: &Path = Path::new(&filename);
            
                        let components: Vec<Component> = path
                            .components()
                            .collect();
            
                        let component: Option<String> = match components.get(0) {
                            Some(component) => {
                                match component {
                                    Component::Normal(s) => {
                                        Some(s.to_str().expect("path should be valid utf-8").to_string())
                                    },
                                    _ => None,
                                }
                            }
                            _ => None,
                        };
            
                        component
                    })
                    .collect();

                let mut uniques = HashSet::new();

                languages.retain(|e| uniques.insert(e.clone()));
            
                languages.insert(0, String::from("en-US"));
                return languages;
            }
        }
    
    };

    gen.into()
}

