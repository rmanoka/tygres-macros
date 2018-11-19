
pub fn trait_impl(input: Struct) -> TokenStream {

    let Struct { ident, generics, is_optional, fields, source } = input;

    let setters: Punctuated<_, Token![,]> = fields.iter()
            .map(Field::as_col_wrapped_ty)
            .collect();

    let refs: Punctuated<_, Token![,]> = fields.iter()
            .map(Field::as_ref_ident)
            .collect();

    let lets: Vec<_> = fields.iter().map(|f| {
        let ident = f.as_ident();
        let ty = f.as_ty();
        let wrapped = f.as_col_wrapped_ty();
        let ref_ident = f.as_ref_ident();

        quote!{ let (#ident, idx): (#ty, usize)
            = <#ty as Makes<'a, #wrapped>>::get(#ref_ident, row, idx); }
    }).collect();

    let getters: Punctuated<_, Token![,]> = fields.iter()
            .map(Field::as_ident)
            .collect();

    let trait_impl = quote!{
        impl<'a> tygres::Makes<'a, Seq![#setters]> for #ident {
            fn get<R: tygres::Row>(s: &'a Seq![#setters], row: &'a R, idx: usize) -> (Self, usize) {
                let ref_seq![#refs] = s;
                #(#lets)*
                (#ident {
                    #getters
                }, idx)

            }
        }
    };
    trait_impl

}

pub fn trait_impl_getter(input: Struct) -> TokenStream {
    let Struct { ident, generics, is_optional, fields, source } = input;
    let src = match source {
        Some(s) => s,
        _ => panic!("source attribute is required for ColumnsSetter"),
    };
    let setters: Punctuated<_, Token![,]> = fields.iter()
            .map(Field::as_col_wrapped_ty)
            .collect();
    let getters: Punctuated<_, Token![,]> = fields.iter()
            .map(Field::as_const)
            .collect();

    quote!{
        impl tygres::Getter for #ident {
            type Src = #src;
            type Sel = Seq![#setters];

            // fn source() -> Self::Src {
            //     #src::default()
            // }
            fn getter() -> Self::Sel {
                seq![#getters]
            }
        }
    }
}

use super::{data::{Struct, Field}};
use proc_macro2::TokenStream;
use inflections::Inflect;
use syn::{
    *, token,
    punctuated::Punctuated,
};
