
extern crate proc_macro;
extern crate syn;
extern crate quote;
extern crate yk_parser_internal;

use proc_macro::TokenStream;
use syn::parse_macro_input;
use yk_parser_internal::{bnf, generate_code};

// Identifier for the front-end lexer library
const FRONT_LIBRARY_NAME: &str = "yk_parser";

#[proc_macro]
pub fn yk_parser(item: TokenStream) -> TokenStream {
     // Identifier for the front-end lexer library
    let FRONT_LIBRARY = quote::format_ident!("{}", FRONT_LIBRARY_NAME);

    // Parse the BNF
    let bnf = parse_macro_input!(item as bnf::RuleSet);
    let code = generate_code(&bnf);

    println!("{}", code);

    code.into()
}
