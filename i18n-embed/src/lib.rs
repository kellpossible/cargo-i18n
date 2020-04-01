extern crate proc_macro;
extern crate proc_macro2;

use crate::proc_macro::TokenStream;

use proc_macro2::{Ident, Span};
use quote::quote;
use syn;

use rust_embed::RustEmbed;

#[proc_macro]
pub fn i18n_embed(input: TokenStream) -> TokenStream {
    input
}

