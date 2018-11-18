#[derive(FromMeta, Debug)]
pub struct Optional;
#[derive(FromMeta, Debug)]
pub struct Required;

pub struct Struct {
    pub ident: Ident,
    pub generics: syn::Generics,
    pub is_optional: bool,
    pub fields: Vec<Field>,
    pub source: Option<Ident>,
}

impl FromDeriveInput for Struct {
    fn from_derive_input(input: &syn::DeriveInput) -> Result<Self, Error> {
        StructReceiver::from_derive_input(input).map(|f| {

            Struct {
                ident: f.ident,
                generics: f.generics,
                is_optional: f.optional.is_some(),
                source: f.source,
                fields: f.data
                            .take_struct()
                            .expect("Should never be enum")
                            .fields.into_iter().map(
                                Into::into
                            ).collect()
            }
        })

    }
}


#[derive(FromDeriveInput, Debug)]
#[darling(attributes(tygres), supports(struct_named))]
pub struct StructReceiver {
    pub ident: Ident,
    pub generics: syn::Generics,

    #[darling(default)]
    pub optional: Option<Optional>,
    #[darling(default)]
    pub source: Option<Ident>,
    pub data: ast::Data<(), FieldReceiver>,
}

#[derive(Debug)]
pub struct Field {
    pub ident: Ident,
    pub ty: Type,
    pub is_optional: Option<bool>,
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
        let col = self.ident.to_string().to_pascal_case();
        let col = Ident::new(&col, self.ident.span());
        quote!{ #col }
    }

    pub fn as_col_wrapped_ty(self: &Self) -> TokenStream {
        let col_ty = self.as_column_ty();
        quote!{ tygres::utils::ColWrap<#col_ty> }
    }

    pub fn as_const(self: &Self) -> TokenStream {
        let col = self.ident.to_string().to_constant_case();
        let ident = Ident::new(&col, self.ident.span());
        quote!{ #ident }
    }

}


use std::result::Result;
impl From<FieldReceiver> for Field {
    fn from(f: FieldReceiver) -> Self {

        let is_optional = f.optional.map(|_| true);

        Field {
            ident: f.ident.expect("Should never be unnamed"),
            ty: f.ty,
            is_optional,
        }

    }
}

#[derive(Debug, FromField)]
#[darling(attributes(tygres))]
pub struct FieldReceiver {
    pub ident: Option<Ident>,
    pub ty: Type,

    #[darling(default)]
    pub optional: Option<Optional>,
}


impl From<DeriveInput> for Struct {
    fn from(input: DeriveInput) -> Struct {
        match FromDeriveInput::from_derive_input(&input) {
            Ok(s) => s,
            Err(e) => {
                panic!(format!("{}", e));
            }
        }
    }
}

use proc_macro2::TokenStream;
use syn::{
    *, token,
    punctuated::Punctuated,
};
use darling::*;
use inflections::*;