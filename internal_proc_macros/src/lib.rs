#[macro_use]
extern crate proc_macro_hack;
#[macro_use]
extern crate quote;
#[macro_use]
extern crate syn;

extern crate regex;
extern crate proc_macro2;


mod guile_impl;

use guile_impl::{parse_guile_impl, parse_guile_defs};
use syn::buffer::TokenBuffer;
use proc_macro2::{TokenStream};
use quote::ToTokens;

proc_macro_item_impl! {
    /// implement a guile func
    pub fn guile_impl_impl(input: &str) -> String {
            let sb = TokenBuffer::new2(
                syn::parse_str::<TokenStream>(input).expect("Turning str into tokenstream"));

            format!("{}", parse_guile_impl(sb.begin())
                    .expect("Expanding guile_impl macro").0)
    }
}

proc_macro_item_impl! {
    /// implement a guile struct
    pub fn guile_defs_impl(input: &str) -> String {
            let sb = TokenBuffer::new2(
                syn::parse_str::<TokenStream>(input).expect("Turning str into tokenstream"));

            let gdefs = parse_guile_defs(sb.begin())
                .expect("Expanding guile_defs macro").0;

            let mut mtokens = quote::Tokens::new();

            gdefs.iter().for_each(
                |gd| gd.construct().to_tokens(&mut mtokens));

            format!("{}", mtokens)
    }
}
