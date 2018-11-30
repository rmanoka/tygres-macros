pub struct Schema {
    tables: Punctuated<Table, Token![,]>,
}

impl Schema {
    pub fn tokens(&self) -> TokenStream {
        let mut t = TokenStream::new();
        let mut idents: HashSet<String> = HashSet::new();
        self.tables.iter()
            .for_each(|tab| t.extend(tab.tokens(&mut idents)));
        t
    }
}

impl Parse for Schema {
    fn parse(input: ParseStream) -> Result<Self> {
        Ok(Schema{
            tables: input.parse_terminated(Table::parse)?
        })
    }
}

pub struct Table {
    name: Ident,
    defined: bool,
    cols: Punctuated<Column, Token![,]>,
}

impl Parse for Table {
    fn parse(input: ParseStream) -> Result<Self> {
        let content;
        Ok(Table{
            defined: {
                let lookahead = input.lookahead1();
                if lookahead.peek(Token![*]) {
                    input.parse::<Token![*]>()?;
                    true
                } else {
                    false
                }},
            name: input.parse()?,
            cols: {
                let lookahead = input.lookahead1();
                if !lookahead.peek(token::Brace) {
                    Punctuated::new()
                } else {
                    braced!(content in input);
                    content.parse_terminated(Column::parse)?
                }
            }
        })
    }
}

impl Table {
    pub fn tokens(&self, idents: &mut HashSet<String>) -> TokenStream {

        let name = &self.name;
        let defined = self.defined
            || idents.contains(&name.to_string());
        idents.insert(name.to_string());
        let snake = name.to_string().to_plural().to_snake_case();

        let mut tokens = if defined {
            quote!{ table!(*#name, #snake); }
        } else {
            quote!{ table!(#name, #snake); }
        };
        tokens.extend(self.cols.iter().map(|c| {
            c.tokens(&self, idents)
        }));
        tokens
    }
}

pub struct Column {
    defined: bool,
    name: Ident,
    col_name: Literal,
    opts: Punctuated<Opt, Token![,]>,
}

impl Parse for Column {
    fn parse(input: ParseStream) -> Result<Self> {
        let lookahead = input.lookahead1();
        let content;

        let defined = if lookahead.peek(Token![*]) {
            input.parse::<Token![*]>()?;
            true
        } else {
            false
        };

        let name: Ident = input.parse()?;
        let col_name = {
            let lookahead = input.lookahead1();
            if !lookahead.peek(token::Paren) {
                Literal::string(&name.to_string().to_snake_case())
            } else {
                let content;
                parenthesized!(content in input);
                let opt: ColName = content.parse()?;
                opt.0
            }
        };

        let opts = {
            let lookahead = input.lookahead1();
            if !lookahead.peek(token::Brace) {
                Punctuated::new()
            } else {
                braced!(content in input);
                content.parse_terminated(Opt::parse)?
            }
        };
        Ok(Column{
            col_name, defined, name, opts,
        })

    }
}

impl Column {
    fn tokens(&self, table: &Table, idents: &mut HashSet<String>) -> TokenStream {

        let t_name = &table.name;
        let name = &self.name;
        let cap = name.to_string().to_screaming_snake_case();
        let cap = Ident::new(&cap, name.span());
        let snake = &self.col_name;

        let defined = self.defined
            || idents.contains(&name.to_string());
        idents.insert(name.to_string());

        let mut tokens = if defined {
            quote!{ column!(#t_name, *#name, #snake); }
        } else {
            quote!{ column!(#t_name, #name, #cap, #snake); }
        };
        tokens.extend(
            self.opts.iter().map(|opt| {
                opt.tokens(&self)
            })
        );
        tokens
    }
}

pub struct ColName(Literal);

impl Parse for ColName {
    fn parse(input: ParseStream) -> Result<Self> {
        let ident: Ident = input.parse()?;
        if ident.eq("column_name") {
            let _eq: Token![=] = input.parse()?;
            let value: Literal = input.parse()?;
            Ok(ColName(value))
        } else {
            Err(input.error(
                format!("Unexpected ident: {}.", ident)
            ))
        }
    }
}


pub enum Opt {
    Takes(Type),
    Makes(Type),
    TakesJson(Type),
    MakesJson(Type),
}

impl Parse for Opt {
    fn parse(input: ParseStream) -> Result<Self> {
        let _lookahead = input.lookahead1();
        let ident: Ident = input.parse()?;
        let _span = ident.span();
        let ident = ident.to_string();
        let content;
        parenthesized!(content in input);

        if ident.eq("takes") {
            Ok(Opt::Takes(content.parse()?))
        } else if ident.eq("takes_json") {
            Ok(Opt::TakesJson(content.parse()?))

        } else if ident.eq("makes") {
            Ok(Opt::Makes(content.parse()?))
        } else if ident.eq("makes_json") {
            Ok(Opt::MakesJson(content.parse()?))

        } else {
            Err(input.error(
                format!("Unexpected ident: {}.", ident)
            ))
        }
    }
}

impl Opt {
    fn tokens(&self, col: &Column) -> TokenStream {
        use Opt::*;
        let name = &col.name;
        match self {
            Takes(ref ty) => quote!{ takes!(#name, #ty); },
            Makes(ref ty) => quote!{ makes!(#name, #ty); },
            TakesJson(ref ty) => quote!{ takes_json!(#name, #ty); },
            MakesJson(ref ty) => quote!{ makes_json!(#name, #ty); },
        }
    }
}

use inflector::Inflector;
use proc_macro2::{TokenStream, Literal};
use syn::{
    Ident, TypeParamBound, Field, Type,
    punctuated::Punctuated,
    token,
    parse::{Parse, ParseStream, Result}
};
use std::collections::HashSet;
