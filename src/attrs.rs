use proc_macro::TokenStream;
use inflections::Inflect;
use syn::{
    *, token,
    punctuated::Punctuated,
    parse::{Parse, ParseStream, Result},
};

pub struct Attrs {
    _paren: token::Paren,
    pub idents: Punctuated<Ident, Token![,]>,
}

impl Parse for Attrs {
    fn parse(input: ParseStream) -> Result<Self> {
        let content;
        Ok(Attrs{
            _paren: parenthesized!(content in input),
            idents: content.parse_terminated(
                Ident::parse
            )?,
        })
    }
}

pub fn parse_attrs(attrs: &[Attribute]) -> Option<Attrs> {
    attrs.iter().find(|attr| {
        (attr.path.segments.len() == 1)
            && (attr.path.segments[0].ident == "tygres")
    }).and_then(|attr| {
        syn::parse2(attr.tts.clone()).ok()
    })
}
