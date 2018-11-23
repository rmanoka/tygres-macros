impl Field {
    fn as_valued_ty(self: &Self, reffed: bool, is_optional: bool) -> TokenStream {
        let f = self;
        let ty = f.as_ty();
        let col = f.as_column_ty();
        let exp = if reffed {
            quote!{(&#ty, #col)}
        } else {
            quote!{(#ty, #col)}
        };
        if f.is_optional.unwrap_or(is_optional) {
            quote!{ Opt!#exp }
        } else {
            quote!{ With!#exp }
        }
    }

    fn as_value_setter(self: &Self, reffed: bool, is_optional: bool) -> TokenStream {
        let f = self;
        let ident = f.as_ident();
        let cap = f.as_const();
        let ident = if reffed {
            quote!{ &self.#ident }
        } else {
            quote!{ self.#ident}
        };
        if f.is_optional.unwrap_or(is_optional) {
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

pub fn trait_impl_ref_setter(input: Struct) -> TokenStream {

    let Struct { ident, generics: _, is_optional, fields, source: _ } = input;

    let types: Punctuated<_, Token![,]> = fields.iter()
        .map(|f| Field::as_valued_ty(f, true, is_optional))
        .collect();

    let setters: Punctuated<_, Token![,]> = fields.iter()
        .map(|f| Field::as_value_setter(f, true, is_optional))
        .collect();

    let trait_impl = quote!{
        impl<'a> tygres::RefSetter<'a> for #ident {
            type Out = Seq![#types];
            fn as_setter(&'a self) -> Self::Out {
                seq![#setters]
            }
        }
    };
    trait_impl

}

pub fn trait_impl_val_setter(input: Struct) -> TokenStream {

    let Struct { ident, generics: _, is_optional, fields, source: _ } = input;

    let types: Punctuated<_, Token![,]> = fields.iter()
        .map(|f| Field::as_valued_ty(f, false, is_optional))
        .collect();

    let setters: Punctuated<_, Token![,]> = fields.iter()
        .map(|f| Field::as_value_setter(f, false, is_optional))
        .collect();

    let trait_impl = quote!{
        impl<'a> tygres::ValSetter for #ident {
            type Out = Seq![#types];
            fn as_setter(self) -> Self::Out {
                seq![#setters]
            }
        }
    };
    trait_impl
}

pub fn trait_impl_takes_unit(input: Struct) -> TokenStream {
    let Struct { ident, generics: _, is_optional, fields, source: _ } = input;

    let pushes: Vec<_> = fields.iter().map(|f| {
        let wrap = f.as_col_wrapped_ty();
        let _ty = f.as_ty();
        let ident = f.as_ident();
        let cap = f.as_const();
        if f.is_optional.unwrap_or(is_optional) {
            quote!{ match self.#ident.as_ref() {
                Some(r) => {
                    <#wrap as tygres::Takes<_>>::push_values(&#cap, r, buf);
                },
                _ => {},
            }}
        } else {
            quote!{ <#wrap as tygres::Takes<_>>::push_values(&#cap, &self.#ident, buf); }
        }
    }).collect();

    quote!{
        impl<'a> tygres::Takes<'a, tygres::utils::Unit> for #ident {
            fn push_values<'b>(&'a self, values: tygres::utils::Unit, buf: &'b mut Vec<&'a postgres::types::ToSql>) {
                #(#pushes)*
            }
        }
    }
}

pub fn trait_impl_columns_setter(input: Struct) -> TokenStream {
    let Struct { ident, generics: _, is_optional, fields, source } = input;
    let src = match source {
        Some(s) => s,
        _ => panic!("source attribute is required for ColumnsSetter"),
    };
    let sels: Punctuated<_, Token![,]> = fields.iter()
        .map(|f| Field::as_value_setter(f, true, is_optional))
        .collect();
    let types: Punctuated<_, Token![,]> = fields.iter()
        .map(|f| Field::as_valued_ty(f, true, is_optional))
        .collect();
    let sels2 = sels.clone();
    quote!{
        impl tygres::ColumnsSetter<#src> for #ident {
            fn push_selection(&self, buf: &mut String) -> bool {
                seq![#(#sels),*].push_selection(buf)
            }
            fn push_values(&self, buf: &mut String, idx: usize) -> usize {
                <Seq![#(#types),*] as tygres::ColumnsSetter<#src>>
                    ::push_values(&seq![#(#sels2),*], buf, idx)
            }
        }
    }
}

pub fn trait_impl_has_setter(input: Struct) -> TokenStream {
    let Struct { ident, generics: _, is_optional: _, fields, source: _ } = input;

    let cols: Punctuated<_, Token![,]> = fields.iter()
        .map(Field::as_const)
        .collect();
    let col_wrapped: Punctuated<_, Token![,]> = fields.iter()
        .map(Field::as_col_wrapped_ty)
        .collect();
    let tys: Punctuated<_, Token![,]> = fields.iter()
        .map(Field::as_ty)
        .collect();
    let idents: Punctuated<_, Token![,]> = fields.iter()
        .map(Field::as_ident)
        .collect();
    let _wrap2 = col_wrapped.clone();
    quote!{
        impl<'a> tygres::HasSetter<'a> for #ident {
            type Val = Seq![#(&'a #tys),*];
            type Set = Seq![#(#col_wrapped),*];

            fn setter() -> Self::Set {
                seq![#(#cols),*]
            }

            fn as_value(&'a self) -> Self::Val {
                seq![#(&self.#idents),*]
            }
        }
    }
}

use super::{
    data::{Field, Struct}
};
use proc_macro2::TokenStream;
use syn::{
    *, token,
    punctuated::Punctuated,
};
