use super::SimpleArg;

use syn::{
    parenthesized,
    parse::{Parse, ParseStream},
    punctuated::Punctuated,
    token::Paren,
    Generics, Ident, ReturnType, Token, WhereClause,
};

pub struct Signature {
    pub fn_token: Token![fn],
    pub ident: Option<Ident>,
    pub generics: Generics,
    pub paren_token: Paren,
    pub inputs: Punctuated<SimpleArg, Token![,]>,
    pub output: ReturnType,
}

fn parse_fn_args(input: ParseStream) -> syn::Result<Punctuated<SimpleArg, Token![,]>> {
    let mut args = Punctuated::new();
    while !input.is_empty() {
        args.push_value(input.parse()?);
        if input.is_empty() {
            break;
        }
        args.push_punct(input.parse()?);
    }

    Ok(args)
}

impl Parse for Signature {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let fn_token: Token![fn] = input.parse()?;
        let ident: Option<Ident> = input.parse()?;
        let generics: Generics = input.parse()?;
        let content;
        let paren_token: Paren = parenthesized!(content in input);
        let inputs: Punctuated<SimpleArg, Token![,]> = parse_fn_args(&content)?;
        let output: ReturnType = input.parse()?;
        let where_clause: Option<WhereClause> = input.parse()?;
        Ok(Signature {
            fn_token,
            ident,
            generics: Generics {
                where_clause,
                ..generics
            },
            paren_token,
            inputs,
            output,
        })
    }
}
