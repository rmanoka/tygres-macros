#![feature(extern_crate_item_prelude, uniform_paths, arbitrary_self_types)]
extern crate proc_macro;
extern crate proc_macro2;

#[macro_use]
extern crate quote;

#[macro_use]
extern crate syn;

mod builder;
mod schema;
mod setter;
mod getter;
mod data;

use self::proc_macro::TokenStream;
use syn::DeriveInput;

#[proc_macro]
pub fn builder(input: TokenStream) -> TokenStream {
    parse_macro_input!(input as builder::Builder)
        .output().into()
}

#[proc_macro]
pub fn schema(input: TokenStream) -> TokenStream {
    use schema::Schema;
    parse_macro_input!(input as Schema)
        .tokens().into()
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
    TokenStream::from(setter::trait_impl_takes_unit(item.into()))
}

#[proc_macro_derive(ColumnsSetter, attributes(tygres))]
pub fn takes_columns_setter_derive(input: TokenStream) -> TokenStream {
    let item = parse_macro_input!(input as DeriveInput);
    TokenStream::from(setter::trait_impl_columns_setter(item.into()))
}

#[proc_macro_derive(Makes, attributes(tygres))]
pub fn makes_derive(input: TokenStream) -> TokenStream {
    let item = parse_macro_input!(input as DeriveInput);
    TokenStream::from(getter::trait_impl(item.into()))
}

#[proc_macro_derive(Getter, attributes(tygres))]
pub fn getter_derive(input: TokenStream) -> TokenStream {
    let item = parse_macro_input!(input as DeriveInput);
    TokenStream::from(getter::trait_impl_getter(item.into()))
}

