
extern crate proc_macro;
extern crate syn;
extern crate quote;

use syn::parse_macro_input;

// Identifier for the front-end lexer library
const FRONT_LIBRARY_NAME: &str = "yk_parser";

#[proc_macro]
pub fn yk_parser(item: TokenStream) -> TokenStream {
     // Identifier for the front-end lexer library
    let FRONT_LIBRARY = quote::format_ident!("{}", FRONT_LIBRARY_NAME);

    // Parse the BNF
    let bnf = parse_macro_input!(item as ::yk_parser::RuleSet);

    TokenStream::new()
}
