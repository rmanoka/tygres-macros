#![feature(extern_crate_item_prelude)]
extern crate proc_macro;
extern crate proc_macro2;

#[macro_use]
extern crate quote;

#[macro_use]
extern crate syn;

mod builder;
mod setter;
mod attrs;

use self::proc_macro::TokenStream;
use syn::DeriveInput;

#[proc_macro]
pub fn builder(input: TokenStream) -> TokenStream {
    let builder = parse_macro_input!(input as builder::Builder);
    let output: TokenStream = builder.output().into();
    output
}

#[proc_macro_derive(Setter, attributes(tygres))]
pub fn setter_derive(input: TokenStream) -> TokenStream {
    let item = parse_macro_input!(input as DeriveInput);

    TokenStream::from(setter::trait_impl(item))
}

