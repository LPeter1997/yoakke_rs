
extern crate proc_macro;
extern crate syn;

use proc_macro::TokenStream;
use syn::{parse_macro_input, ItemEnum};

struct Token<'a, T> {
    pub kind: T,
    pub value: &'a str,
}

#[proc_macro_derive(Lexer, attributes(
    error,
    end,
    c_ident,
    regex,
))]
pub fn yk_lexer(item: TokenStream) -> TokenStream {
    // Parse the enum
    let enm = parse_macro_input!(item as ItemEnum);

    // Things we need to fill
    let mut err_variant = None;
    let mut end_variant = None;

    for variant in &enm.variants {
        let var_attrs = &variant.attrs;

        for attr in var_attrs {
            let ident = &attr.path.segments[0].ident;

            if ident == "error" {
                if err_variant.is_some() {
                    panic!("You can only define one 'error' variant!");
                }

                err_variant = Some(0);
            }
            else if ident == "end" {
                if end_variant.is_some() {
                    panic!("You can only define one 'end' variant!");
                }

                end_variant = Some(0);
            }
            else if ident == "c_ident" {

            }
            else if ident == "regex" {

            }
            else {
                // TODO: Do we panic?
            }
        }
    }

    if err_variant.is_none() {
        panic!("You must define an 'error' variant!");
    }

    if end_variant.is_none() {
        panic!("You must define an 'end' variant!");
    }

    unimplemented!();
}

#[derive(Lexer)]
pub enum TokenType {

}
