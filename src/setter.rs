impl Field {
    fn as_valued_ty(self: &Self, reffed: bool, def_optional: bool) -> TokenStream {
        let f = self;
        let ty = f.as_ty();
        let col = f.as_column_ty();
        let exp = if reffed {
            quote!{(&'a #ty, #col)}
        } else {
            quote!{(#ty, #col)}
        };
        if f.is_optional.unwrap_or(def_optional) {
            quote!{ Opt!#exp }
        } else {
            quote!{ With!#exp }
        }
    }

    fn as_value_setter(self: &Self, reffed: bool, def_optional: bool) -> TokenStream {
        let f = self;
        let ident = f.as_ident();
        let cap = f.as_const();
        let ident = if reffed {
            quote!{ &self.#ident }
        } else {
            quote!{ self.#ident}
        };
        if f.is_optional.unwrap_or(def_optional) {
            if reffed {
                quote!{ #cap.if_some_ref(#ident) }
            } else {
                quote!{ #cap.if_some(#ident) }
            }
        } else {
            quote!{ #cap.taking(#ident) }
        }
    }
}

pub fn trait_impl(input: Struct) -> TokenStream {

    let Struct { ident, generics, is_optional, fields } = input;

    let types: Punctuated<_, Token![,]> = fields.iter()
        .map(|f| Field::as_valued_ty(f, true, is_optional))
        .collect();

    let setters: Punctuated<_, Token![,]> = fields.iter()
        .map(|f| Field::as_value_setter(f, true, is_optional))
        .collect();

    let trait_impl = quote!{
        impl<'a> tygres::Setter<'a> for #ident {
            type Out = Seq![#types];
            fn as_setter(&'a self) -> Self::Out {
                seq![#setters]
            }
        }
    };
    trait_impl

}

pub fn trait_impl_owned(input: Struct) -> TokenStream {

    let Struct { ident, generics, is_optional, fields } = input;

    let types: Punctuated<_, Token![,]> = fields.iter()
        .map(|f| Field::as_valued_ty(f, false, is_optional))
        .collect();

    let setters: Punctuated<_, Token![,]> = fields.iter()
        .map(|f| Field::as_value_setter(f, false, is_optional))
        .collect();

    let trait_impl = quote!{
        impl<'a> tygres::OwnedSetter for #ident {
            type Out = Seq![#types];
            fn as_setter(self) -> Self::Out {
                seq![#setters]
            }
        }
    };
    trait_impl
}

pub fn trait_impl_takes_unit(input: Struct) -> TokenStream {
    let Struct { ident, generics, is_optional, fields } = input;
    let types: Vec<_> = fields.iter()
        .map(|f| Field::as_valued_ty(f, true, is_optional))
        .collect();

    let setters: Vec<_> = fields.iter()
        .map(|f| Field::as_value_setter(f, true, is_optional))
        .collect();

    quote!{
        impl<'a> tygres::Takes<'a, tygres::utils::Unit> for #ident {
            fn push_values<'b>(&'a self, values: Unit, buf: &'b mut Vec<&'a postgres::types::ToSql>) {
                #(<#types as Takes<'a, Unit>>::push_values(&#setters, values, buf);)*
            }
        }
    }
}

use super::{
    data::{Field, Struct}
};
use proc_macro2::TokenStream;
use inflections::Inflect;
use syn::{
    *, token,
    punctuated::Punctuated,
};
