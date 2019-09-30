/**
 * Parenthesis and bracket utilities I missed from the 'syn' crate.
 */

use syn::Result;
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
