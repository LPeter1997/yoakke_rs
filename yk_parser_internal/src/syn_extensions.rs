/**
 * Parenthesis, bracket and other utilities I missed from the 'syn' crate.
 */

use syn::{Result, Ident, Error};
use syn::parse::ParseStream;
use syn::token::{Paren, Bracket};

pub fn parse_parenthesized_fn<F, T>(input: ParseStream, mut parser: F)
    -> Result<(Paren, T)> where F : FnMut(ParseStream) -> Result<T> {

    let content;
    let paren = syn::parenthesized!(content in input);
    let inner = parser(&content)?;
    Ok((paren, inner))
}

pub fn parse_bracketed_fn<F, T>(input: ParseStream, mut parser: F)
    -> Result<(Bracket, T)> where F : FnMut(ParseStream) -> Result<T> {

    let content;
    let paren = syn::bracketed!(content in input);
    let inner = parser(&content)?;
    Ok((paren, inner))
}

pub fn parse_ident(input: ParseStream, id_content: &str) -> Result<Ident> {
    let id: Ident = input.parse()?;
    let id_str = id.to_string();
    if id_str != id_content {
        Err(Error::new(id.span(),
            format!("Expected identifier '{}', but got '{}'!", id_content, id_str)))
    }
    else {
        Ok(id)
    }
}
