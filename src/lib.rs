#![feature(extern_crate_item_prelude, uniform_paths, arbitrary_self_types)]
extern crate proc_macro;
extern crate proc_macro2;

#[macro_use]
extern crate quote;

#[macro_use]
extern crate syn;

mod builder;
mod setter;
mod getter;
mod data;

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

    TokenStream::from(setter::trait_impl(item.into()))
}

#[proc_macro_derive(OwnedSetter, attributes(tygres))]
pub fn setter_owned_derive(input: TokenStream) -> TokenStream {
    let item = parse_macro_input!(input as DeriveInput);
    TokenStream::from(setter::trait_impl_owned(item.into()))
}

#[proc_macro_derive(TakesUnit, attributes(tygres))]
pub fn takes_unit_derive(input: TokenStream) -> TokenStream {
    let item = parse_macro_input!(input as DeriveInput);
    let t = TokenStream::from(setter::trait_impl_takes_unit(item.into()));
    println!("{}", t);
    t
}

#[proc_macro_derive(Getter, attributes(tygres))]
pub fn getter_derive(input: TokenStream) -> TokenStream {
    let item = parse_macro_input!(input as DeriveInput);
    TokenStream::from(getter::trait_impl(item.into()))
}

