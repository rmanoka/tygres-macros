#[derive(Default)]
pub struct Opts {
    pub optional: bool,
    pub column_name: Option<String>,
    pub wrap: Option<Ident>,
    pub source: Option<Ident>,
}

pub enum TygresOption {
    Optional,
    ColumnName(Literal),
    Wrap(Ident),
    Source(Ident),
}

impl Parse for TygresOption {
    fn parse(input: ParseStream) -> Result<Self> {
        let ident: Ident = input.parse()?;
        let lookahead = input.lookahead1();

        fn parse_val(input: &ParseStream) -> Result<Literal> {
            let eq: Token![=] = input.parse()?;
            input.parse()
        }

        if ident.eq("column_name") {
            Ok(TygresOption::ColumnName(
                parse_val(&input)?))
        } else if ident.eq("wrap") {
            let val = parse_val(&input)?.to_string();
            let val = val.split('"').nth(1).unwrap();
            Ok(TygresOption::Wrap(Ident::new(&val, ident.span())))
        } else if ident.eq("source") {
            let val = parse_val(&input)?.to_string();
            let val = val.split('"').nth(1).unwrap();
            Ok(TygresOption::Source(Ident::new(&val, ident.span())))
        } else if ident.eq("optional") {
            Ok(TygresOption::Optional)
        } else {
            Err(input.error("unknown option"))
        }
    }
}

pub struct TygresOptions {
    opts: Punctuated<TygresOption, Token![,]>
}

impl Parse for TygresOptions {
    fn parse(input: ParseStream) -> Result<Self> {
        let content;
        parenthesized!(content in input);
        Ok(TygresOptions {
            opts: content.parse_terminated(TygresOption::parse)?,
        })
    }
}

pub fn parse_attribute(attrs: &Vec<Attribute>) -> Opts {
    let mut out = Opts::default();
    let attr = attrs.iter().find(|attr| {
        let path = &attr.path;
        if (path.segments.len() != 1) { return false; }
        let seg = path.segments[0].ident.to_string();
        seg.eq("tygres")
    });
    if let Some(attr) = attr {
        parse_tygres_attribute(attr, &mut out);
    }
    out
}

pub fn parse_tygres_attribute(attr: &Attribute, def: &mut Opts) {
    let tts = &attr.tts;
    let opts: TygresOptions = match syn::parse2(tts.to_owned()) {
        Ok(o) => o,
        Err(e) => panic!(format!("{}", e)),
    };
    opts.opts.into_iter().for_each(|opt| {
        use TygresOption::*;
        match opt {
            Optional => { def.optional = true; },
            ColumnName(s) => { def.column_name = Some(s.to_string()); },
            Wrap(s) => { def.wrap = Some(s); },
            Source(s) => { def.source = Some(s); },
        }
    });
}

use proc_macro2::{TokenStream, Literal};
use syn::{
    *, Ident, TypeParamBound, Field, Type,
    punctuated::Punctuated,
    token,
    parse::{Parse, ParseStream, Result}
};
