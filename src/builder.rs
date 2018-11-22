use proc_macro2::TokenStream;
use syn::{
    Ident, TypeParamBound, Field, Type,
    punctuated::Punctuated,
    token,
    parse::{Parse, ParseStream, Result}
};

pub struct Builder {
    _struct_token: Token![struct],
    name: Ident,
    _brace: token::Brace,
    type_fields: Punctuated<TypeField, Token![,]>,
}

impl Builder {

    pub fn output(&self) -> TokenStream {

        let fields_ref = &self.type_fields;

        let name = &self.name;
        let bounds = {
            let bounds = fields_ref.into_iter().map(|f| {
                f.with_bounds()
            });
            quote!(#(#bounds),*)
        };
        let fields = {
            fields_ref.into_iter().map(|f| {
                let field_name = &f.field_name;
                let type_name = &f.type_name;
                quote!(pub #field_name: #type_name)
            })
        };
        let struct_def = quote!(
            pub struct #name<#bounds> {
                #(#fields),*
            }
        );

        let types: Vec<Ident> = fields_ref.into_iter().map(|f| {
            f.type_name.clone()
        }).collect();

        let types_spec = {
            let t_ref = &types;
            quote!(#(#t_ref),*)
        };

        let setters = {
            let mut idx = 0;
            fields_ref.into_iter().map(move |f| {
                idx += 1;
                match f.setter {
                    Some(ref setter) => setter.as_fn(&self, &types, idx - 1),
                    None => TokenStream::new(),
                }
            })
        };

        let _fields = {
            fields_ref.into_iter().map(|f| {
                &f.field_name
            })
        };

        quote!(
            #struct_def
            impl<#bounds> #name<#types_spec> {
                #(#setters)*
            }
        ).into()
    }
}

impl Parse for Builder {
    fn parse(input: ParseStream) -> Result<Self> {
        let content;
        Ok(Builder{
            _struct_token: input.parse()?,
            name: input.parse()?,
            _brace: braced!(content in input),
            type_fields: content.parse_terminated(TypeField::parse)?,
        })
    }
}

pub struct TypeField {
    type_name: Ident,
    colon: Token![:],
    field_name: Ident,
    bounds: Option<Punctuated<TypeParamBound, token::Add>>,
    setter: Option<Setter>,
}

impl TypeField {
    fn with_bounds(&self) -> TokenStream {
        let type_name = &self.type_name;
        let bounds = match self.bounds {
            None => quote!(),
            Some(ref b) => quote!(: #b),
        };
        quote!(#type_name#bounds).into()
    }
}

impl Parse for TypeField {
    fn parse(input: ParseStream) -> Result<Self> {
        let mut field = TypeField {
            field_name: input.parse()?,
            colon: input.parse()?,
            type_name: input.parse()?,
            bounds: None,
            setter: None,
        };
        let lookahead = input.lookahead1();
        if lookahead.peek(token::Bracket) {
            let content;
            bracketed!(content in input);
            field.bounds = Some(content.parse_terminated(TypeParamBound::parse)?);
        }
        if lookahead.peek(Token![as]) {
            field.setter = Some(input.parse()?)
        }
        Ok(field)
    }
}

struct Setter {
    _as_token: Token![as],
    star: Option<Token![*]>,
    name: Ident,
    _paren: token::Paren,
    type_name: Ident,
    bounds: Option<Punctuated<TypeParamBound, token::Add>>,
}

impl Setter {
    fn as_fn(&self, builder: &Builder, types: &[Ident], idx: usize) -> TokenStream {

        let type_field = &builder.type_fields[idx];

        let method_name = &self.name;
        let type_name = &self.type_name;
        let field_name = &type_field.field_name;
        let builder_name = &builder.name;

        let bounds = match self.bounds {
            None => quote!{},
            Some(ref b) => {
                quote!{: #b}
            }
        };

        let types = {
            let mut i = 0;
            types.into_iter().map(move |t| {
                i += 1;
                if idx == (i - 1) {
                    match self.star {
                        Some(_) => quote!(#type_name),
                        None => quote!(Wrap<#type_name>),
                    }

                } else {
                    quote!(#t)
                }
            })
        };

        let fields_ref = &builder.type_fields;
        let fields = {
            let mut i = 0;
            fields_ref.into_iter().map(move |f| {
                i += 1;
                let field_name = &f.field_name;
                let _type_name = &f.type_name;
                if idx == (i - 1) {
                    match self.star {
                        Some(_) => quote!(#field_name: #field_name),
                        None => quote!(#field_name: Wrap(#field_name)),
                    }
                } else {
                    quote!(#field_name: self.#field_name)
                }
            })
        };


        let output = quote!{
            pub fn #method_name <#type_name#bounds>(self, #field_name: #type_name)
            -> #builder_name<#(#types),*> {
                #builder_name {
                    #(#fields),*
                }
            }
        };
        output.into()
    }
}

impl Parse for Setter {
    fn parse(input: ParseStream) -> Result<Self> {
        let content;
        let mut setter = Setter {
            _as_token: input.parse()?,
            star: input.parse()?,
            name: input.parse()?,
            _paren: parenthesized!(content in input),
            type_name: content.parse()?,
            bounds: None,
        };
        let lookahead = content.lookahead1();
        if lookahead.peek(token::Colon) {
            let _ = content.parse::<Token![:]>()?;
            setter.bounds = Some(content.parse_terminated(TypeParamBound::parse)?)
        }
        Ok(setter)
    }
}

