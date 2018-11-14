use super::{attrs};
use proc_macro2::TokenStream;
use inflections::Inflect;
use syn::{
    *, token,
    punctuated::Punctuated,
};

struct Field {
    ident: Ident,
    ty: Type,
    is_optional: bool,
}


impl From<syn::Field> for Field {
    fn from(f: syn::Field) -> Field {
        let is_optional = attrs::parse_attrs(&f.attrs).and_then(|attrs| {
            attrs.idents.into_iter().find(|ident| {
                ident.eq("optional")
            })
        }).is_some();
        Field {
            ident: f.ident.unwrap(),
            ty: f.ty,
            is_optional,
        }
    }
}


pub fn trait_impl(input: DeriveInput) -> TokenStream {

    let ident = input.ident;
    let fields = match input.data {
        Data::Struct(s) => s.fields,
        _ => panic!("Only struct items are supported."),
    };

    let fields = match(fields) {
        Fields::Named(f) => f.named,
        _ => panic!("Only named struct items are supported."),
    };

    let fields: Punctuated<Field, Token![,]> = fields.into_iter().map(|f: syn::Field| {
        Into::<Field>::into(f)
    }).collect();

    let types: Punctuated<_, Token![,]> = fields.iter().map(|f| {
        let ty = f.ty.clone();
        let ident = &f.ident.clone();
        let col = ident.to_string().to_pascal_case();
        let col = Ident::new(&col, ident.span());
        let exp = quote!{(&'a #ty, #col)};
        if f.is_optional {
            quote!{ Opt!#exp }
        } else {
            quote!{ With!#exp }
        }
    }).collect();

    let setters: Punctuated<_, Token![,]> = fields.iter().map(|f| {
        let ident = f.ident.clone();
        let cap = ident.to_string().to_constant_case();
        let cap = Ident::new(&cap, ident.span());
        if f.is_optional {
            quote!{ #cap.if_some_ref(&self.#ident) }
        } else {
            quote!{ #cap.taking(&self.#ident) }
        }
    }).collect();

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