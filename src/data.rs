pub struct Struct {
    pub ident: Ident,
    pub generics: syn::Generics,
    pub optional: bool,
    pub fields: Vec<Field>,
    pub source: Option<Ident>,
}

impl From<DeriveInput> for Struct {
    fn from(input: DeriveInput) -> Struct {
        let opts = parse_attribute(&input.attrs);

        Struct {
            ident: input.ident,
            generics: input.generics,
            fields: match input.data {
                syn::Data::Struct(s) => { match s.fields {
                    syn::Fields::Named(f) => {
                        f.named.into_iter().map(Into::into).collect()
                    },
                    _ => panic!("Only named structs supported!"),
                }},
                _ => panic!("Only named structs supported!"),
            },
            optional: opts.optional,
            source: opts.source,
        }
    }
}


#[derive(Debug)]
pub struct Field {
    pub ident: Ident,
    pub column_name: String,
    pub ty: Type,
    pub optional: bool,
    pub wrap: Option<Ident>,
}

impl Field {
    pub fn as_ident(self: &Self) -> TokenStream {
        let ident = self.ident.to_string();
        let ident = Ident::new(&ident, self.ident.span());
        quote!{ #ident }
    }

    pub fn as_ref_ident(self: &Self) -> TokenStream {
        let mut ident = self.ident
            .to_string().to_owned();
        ident += "_ref";
        let ident = Ident::new(&ident, self.ident.span());
        quote!{ #ident }
    }

    pub fn as_ty(self: &Self) -> TokenStream {
        let ty = self.ty.clone();
        quote!{ #ty }
    }

    pub fn as_column_ty(self: &Self) -> TokenStream {
        let col = self.column_name.to_pascal_case();
        let col = Ident::new(&col, self.ident.span());
        quote!{ #col }
    }

    pub fn as_col_wrapped_ty(self: &Self) -> TokenStream {
        let col_ty = self.as_column_ty();
        quote!{ tygres::utils::ColWrap<#col_ty> }
    }

    pub fn as_const(self: &Self) -> TokenStream {
        let col = self.column_name.to_screaming_snake_case();
        let ident = Ident::new(&col, self.ident.span());
        quote!{ #ident }
    }

}


impl From<syn::Field> for Field {
    fn from(input: syn::Field) -> Field {

        let opts = parse_attribute(&input.attrs);
        let ident = input.ident.unwrap();
        let column_name = opts.column_name.unwrap_or_else(|| ident.to_string());

        Field {
            ident, column_name,
            ty: input.ty,
            optional: opts.optional,
            wrap: opts.wrap,
        }
    }
}


use proc_macro2::TokenStream;
use syn::{
    *, token,
    punctuated::Punctuated,
};
use inflector::Inflector;
use super::attrs::*;
